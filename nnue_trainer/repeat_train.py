import json
from math import sqrt
import shutil
import subprocess
from pathlib import Path

import torch
from checkpoint import load_checkpoint, rotate_and_save
from config import load_config, Config
from dataset import create_dataloader
from model import NNUE
from trainer import DEVICE, train

# Logging strategy: log before each operation, like file creation/deletion/epoch

# Error/SIGINT handling: will crash script; generated positions in chunk folder won't automatically be recovered 
# unless done manually which you can because chunks still there, chunk folder auto-erased next run; 
# if in saving stage then SIGINT is protected against, other signals or unexpected error is undefined behaviour.

SCRIPT_DIR = Path(__file__).parent.resolve()
GENERATE_PARALLEL_PATH = SCRIPT_DIR / "generate_parallel.sh"
CONCAT_CHUNKS_PATH = SCRIPT_DIR / "concat_chunks.sh"
TRAINING_BIN_PATH = SCRIPT_DIR / "training.bin"

def validate(model: NNUE, val_path: str, sigmoid_scale: float, batch_size: int = 1024) -> float:
    print("Validating")
    model.eval()
    dataloader = create_dataloader(val_path, sigmoid_scale, batch_size=batch_size, shuffle=False, num_workers=0)
    total_loss = 0.0
    count = 0
    with torch.no_grad():
        for indices_stm, offsets_stm, indices_opp, offsets_opp, scores in dataloader:
            indices_stm = indices_stm.to(DEVICE)
            offsets_stm = offsets_stm.to(DEVICE)
            indices_opp = indices_opp.to(DEVICE)
            offsets_opp = offsets_opp.to(DEVICE)
            scores = scores.to(DEVICE)
            pred = model(indices_stm, offsets_stm, indices_opp, offsets_opp)
            pred_sig = torch.sigmoid(pred / sigmoid_scale)
            loss = torch.nn.functional.mse_loss(pred_sig, scores)
            total_loss += loss.item()
            count += 1
    avg = total_loss / count if count > 0 else float("inf")
    model.train()
    print(f"Validation loss: {avg:.4f}, square root: {sqrt(avg):.4f}")
    return avg

def bin_output_path_to_chunk_folder(bin_output_path: Path) -> Path:
    return bin_output_path.with_suffix(".chunks")

def generate_positions_bin(bin_output_path: Path, config: Config, use_games_per_worker_validation_set: bool):
    chunk_dir = bin_output_path_to_chunk_folder(bin_output_path)
    # Clean up last iterations positions on start of new iteration
    if chunk_dir.exists():
        print(f"Removing chunk directory {chunk_dir}")
        shutil.rmtree(chunk_dir)
    if bin_output_path.exists():
        print(f"Removing output bin {bin_output_path}")
        bin_output_path.unlink()
    gen_args = [
        str(GENERATE_PARALLEL_PATH), str(chunk_dir), str(config["workers"]),
        "--num-games", str(config["games_per_worker_validation_set"] if use_games_per_worker_validation_set else config["games_per_worker"]),
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

    checkpoint = load_checkpoint(config)
    if checkpoint.cycle > 0:
        print(f"Resuming from cycle {checkpoint.cycle}")

    if not Path(val_path_str).exists():
        print(f"Validation file {val_path_str} not found, generating...")
        val_path = Path(val_path_str)
        generate_positions_bin(val_path, config, True)
        print("Removing validation chunks folder")
        shutil.rmtree(bin_output_path_to_chunk_folder(val_path))

    track_path = Path(val_path_str).with_suffix('.track.json')

    for cycle in range(checkpoint.cycle + 1, config["cycles"] + 1):
        print(f"Begin parallel generating positions {cycle}/{config['cycles']}")
        generate_positions_bin(TRAINING_BIN_PATH, config, False)
        train(
            str(TRAINING_BIN_PATH),
            epochs_per_cycle,
            checkpoint.model,
            checkpoint.optimizer,
            batch_size,
            sigmoid_scale,
        )
        checkpoint.cycle = cycle

        rmse = None
        if cycle % val_every == 0:
            rmse = sqrt(validate(checkpoint.model, val_path_str, sigmoid_scale))

        rotate_and_save(config, checkpoint)
        if rmse is not None:
            history = json.loads(track_path.read_text()) if track_path.exists() else []
            history.append(rmse)
            track_path.write_text(json.dumps(history))

        print(f"Cycle {cycle} done")
    print("Done")

if __name__ == "__main__":
    repeat_train(load_config())
