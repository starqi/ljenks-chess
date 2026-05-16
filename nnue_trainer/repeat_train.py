import json
from math import sqrt
import os
import shutil
import signal
import subprocess
from pathlib import Path

import torch
from config import load_config, Config
from dataset import create_dataloader
from model import NNUE
from trainer import get_device, train

# Logging strategy: log before each operation, like file creation/deletion/epoch

# Error/SIGINT handling: will crash script; generated positions in chunk folder won't automatically be recovered 
# unless done manually which you can because chunks still there, chunk folder auto-erased next run; 
# if in saving stage then SIGINT is protected against, other signals or unexpected error is undefined behaviour.

SCRIPT_DIR = Path(__file__).parent.resolve()
GENERATE_PARALLEL_PATH = SCRIPT_DIR / "generate_parallel.sh"
CONCAT_CHUNKS_PATH = SCRIPT_DIR / "concat_chunks.sh"
TRAINING_BIN_PATH = SCRIPT_DIR / "training.bin"

def validate(model: NNUE, val_path: str, device: torch.device, batch_size: int = 2048) -> float:
    print("Validating")
    model.eval()
    dataloader = create_dataloader(val_path, batch_size=batch_size, shuffle=False, num_workers=0)
    total_loss = 0.0
    count = 0
    with torch.no_grad():
        for indices_stm, offsets_stm, indices_opp, offsets_opp, scores in dataloader:
            indices_stm = indices_stm.to(device)
            offsets_stm = offsets_stm.to(device)
            indices_opp = indices_opp.to(device)
            offsets_opp = offsets_opp.to(device)
            scores = scores.to(device)
            pred = model(indices_stm, offsets_stm, indices_opp, offsets_opp)
            loss = torch.nn.functional.mse_loss(pred, scores)
            total_loss += loss.item()
            count += 1
    avg = total_loss / count if count > 0 else float("inf")
    model.train()
    print(f"Validation loss: {avg:.4f}, square root: {sqrt(avg):.4f}")
    return avg

def _save_and_rotate(model: NNUE, model_path: Path, checkpoint_backup_count: int, rmse_to_record: tuple[float, Path] | None) -> None:
    old_handler = signal.signal(signal.SIGINT, signal.SIG_IGN)
    try:
        oldest = Path(f"{model_path}.bak.{checkpoint_backup_count}")
        if oldest.exists():
            print(f"Deleting {oldest}")
            oldest.unlink()
        for i in range(checkpoint_backup_count - 1, 0, -1):
            src = Path(f"{model_path}.bak.{i}")
            if src.exists():
                src_str = str(src)
                new_str = f"{model_path}.bak.{i + 1}"
                print(f"{src_str} -> {new_str}")
                os.replace(src_str, new_str)
        if model_path.exists():
            src_str = str(model_path)
            new_str = f"{model_path}.bak.1"
            print(f"{src_str} -> {new_str}")
            os.replace(src_str, new_str)
        model_path_str = str(model_path)
        print(f"Saving to {model_path_str}")
        torch.save(model.state_dict(), model_path_str)
        if rmse_to_record is not None:
            track_path = rmse_to_record[1]
            history = json.loads(track_path.read_text()) if track_path.exists() else []
            history.append(rmse_to_record[0])
            track_path.write_text(json.dumps(history))
    finally:
        signal.signal(signal.SIGINT, old_handler)

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
    batch_size = config["batch_size"]
    lr = config["lr"]
    model_path_str = config["checkpoint_path"]

    cycles = config["cycles"]
    epochs_per_cycle = config["epochs_per_cycle"]
    val_every = config["val_every"]
    val_path_str = config["validation_path"]

    device = get_device()
    model = NNUE().to(device)
    if os.path.exists(model_path_str):
        # weights_only = Non-executable data only, legacy pickler BS  
        model.load_state_dict(torch.load(model_path_str, map_location=device, weights_only=True))
        print(f"Loaded checkpoint from {model_path_str}")

    if not os.path.exists(val_path_str):
        print(f"Validation file {val_path_str} not found, generating...")
        val_path = Path(val_path_str)
        generate_positions_bin(val_path, config, True)
        print("Removing validation chunks folder")
        shutil.rmtree(bin_output_path_to_chunk_folder(val_path))

    track_path = Path(val_path_str).with_suffix('.track.json')

    for cycle in range(1, cycles + 1):
        print(f"Begin parallel generating positions {cycle}/{cycles}")
        generate_positions_bin(TRAINING_BIN_PATH, config, False)
        model = train(
            str(TRAINING_BIN_PATH),
            epochs_per_cycle,
            model,
            batch_size,
            lr
        )
        rmse = None
        if cycle % val_every == 0:
            rmse = (sqrt(validate(model, val_path_str, device)), track_path)
        _save_and_rotate(model, Path(model_path_str), config["checkpoint_backup_count"], rmse_to_record=rmse)
        print(f"Cycle {cycle} done")
    print("Done")

if __name__ == "__main__":
    repeat_train(load_config())
