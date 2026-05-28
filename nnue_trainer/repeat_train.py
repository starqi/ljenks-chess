import json
from math import sqrt
import shutil
import subprocess
from pathlib import Path

from checkpoint import load_checkpoint, rotate_and_save
from config import load_config, Config
from trainer import validate, train

# Logging strategy: log before each operation, like file creation/deletion/epoch

# Error/SIGINT handling: will crash script; generated positions in chunk folder won't automatically be recovered 
# unless done manually which you can because chunks still there, chunk folder auto-erased next run; 
# if in saving stage then SIGINT is protected against, other signals or unexpected error is undefined behaviour.

SCRIPT_DIR = Path(__file__).parent.resolve()
GENERATE_PARALLEL_PATH = SCRIPT_DIR / "generate_parallel.sh"
CONCAT_CHUNKS_PATH = SCRIPT_DIR / "concat_chunks.sh"
COUNT_POSITIONS_PATH = SCRIPT_DIR / "count_positions.sh"
TRAINING_BIN_PATH = SCRIPT_DIR / "training.bin"

MOVES_PER_GAME = 100


def bin_output_path_to_chunk_folder(bin_output_path: Path) -> Path:
    return bin_output_path.with_suffix(".chunks")

def _count_positions_in_chunks(chunk_dir: Path) -> int:
    result = subprocess.run(
        [str(COUNT_POSITIONS_PATH), str(chunk_dir)],
        capture_output=True, text=True, check=True,
    )
    return int(result.stdout.strip())

def generate_positions_bin(bin_output_path: Path, config: Config, use_games_per_worker_validation_set: bool):
    chunk_dir = bin_output_path_to_chunk_folder(bin_output_path)
    bin_exists = bin_output_path.exists()
    chunks_exist = chunk_dir.exists() and any(chunk_dir.glob("*.bin"))

    games_per_worker = (config["games_per_worker_validation_set"]
                        if use_games_per_worker_validation_set
                        else config["games_per_worker"])

    skip_gen_concat_only = False
    if bin_exists:
        print(f"Bin file {bin_output_path} exists, cleaning up for fresh generation")
        if chunk_dir.exists():
            print(f"Removing chunk directory {chunk_dir}")
            shutil.rmtree(chunk_dir)
        print(f"Removing output bin {bin_output_path}")
        bin_output_path.unlink()
    elif chunks_exist:
        existing_count = _count_positions_in_chunks(chunk_dir)
        intended_positions = games_per_worker * config["workers"] * MOVES_PER_GAME
        remaining_positions = intended_positions - existing_count
        print(f"Resuming interrupted generation: {existing_count}/{intended_positions} positions exist")

        if remaining_positions <= 0:
            print(f"Already have enough positions, skipping generation, concat'ing for final bin file")
            skip_gen_concat_only = True
        else:
            games_per_worker = max(1, remaining_positions // (config["workers"] * MOVES_PER_GAME))
            print(f"Generating ~{remaining_positions} more positions ({games_per_worker} games/worker)")
    else:
        print(f"No existing bin or chunks, starting fresh generation")

    if not skip_gen_concat_only:
        gen_args = [
            str(GENERATE_PARALLEL_PATH), str(chunk_dir), str(config["workers"]),
            "--num-games", str(games_per_worker),
            "--max-nodes", str(config["max_nodes"]),
            "--random-half-moves", str(config["random_half_moves"]),
            "--max-half-moves-per-game", str(config["max_half_moves"]),
        ]
        print("Running position gen scripts")
        subprocess.run(gen_args, check=True)
    subprocess.run([str(CONCAT_CHUNKS_PATH), str(chunk_dir), str(bin_output_path)], check=True)

def repeat_train(config: Config):
    batch_size: int = config["batch_size"]
    epochs_per_cycle: int = config["epochs_per_cycle"]
    val_every: int = config["val_every"]
    val_path_str: str = config["validation_path"]
    sigmoid_scale: float = config["sigmoid_scale"]
    loader_num_workers: int = config["loader_num_workers"]

    # Cycle, model, optimizer will change over time
    mutable_checkpoint = load_checkpoint(config)
    if mutable_checkpoint.cycle > 0:
        print(f"Resuming from cycle {mutable_checkpoint.cycle}")

    if not Path(val_path_str).exists():
        print(f"Validation file {val_path_str} not found, generating...")
        val_path = Path(val_path_str)
        generate_positions_bin(val_path, config, True)
        print("Removing validation chunks folder")
        shutil.rmtree(bin_output_path_to_chunk_folder(val_path))

    track_path = Path(val_path_str).with_suffix('.track.json')

    for cycle in range(mutable_checkpoint.cycle + 1, config["cycles"] + 1):
        print(f"Begin parallel generating positions {cycle}/{config['cycles']}")
        generate_positions_bin(TRAINING_BIN_PATH, config, False)
        mutable_checkpoint.cycle = cycle
        train(
            str(TRAINING_BIN_PATH),
            epochs_per_cycle,
            mutable_checkpoint.model,
            mutable_checkpoint.optimizer,
            batch_size,
            sigmoid_scale,
            loader_num_workers,
            save_callback=lambda: rotate_and_save(config, mutable_checkpoint),
        )

        rmse = None
        if cycle % val_every == 0:
            rmse = sqrt(validate(
                mutable_checkpoint.model,
                val_path_str,
                sigmoid_scale,
                batch_size,
                loader_num_workers
            ))
        if rmse is not None:
            history = json.loads(track_path.read_text()) if track_path.exists() else []
            history.append(rmse)
            track_path.write_text(json.dumps(history))

        print(f"Cycle {cycle} done")
    print("Done")

if __name__ == "__main__":
    repeat_train(load_config())
