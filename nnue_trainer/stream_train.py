import os
import shutil
import signal
import subprocess
import time
from pathlib import Path

import torch
from config import load_config, Config
from dataset import create_dataloader
from model import NNUE
from trainer import get_device, train

_SCRIPT_DIR = Path(__file__).parent.resolve()

def validate(model: NNUE, val_path: str, device: torch.device, batch_size: int = 2048) -> float:
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
    return avg

def _save_and_rotate(model: NNUE, model_path: Path, backup_count: int) -> None:
    old_handler = signal.signal(signal.SIGINT, signal.SIG_IGN)
    try:
        oldest = Path(f"{model_path}.bak.{backup_count}")
        if oldest.exists():
            oldest.unlink()
        for i in range(backup_count - 1, 0, -1):
            src = Path(f"{model_path}.bak.{i}")
            if src.exists():
                os.replace(str(src), f"{model_path}.bak.{i + 1}")
        if model_path.exists():
            os.replace(str(model_path), f"{model_path}.bak.1")
        torch.save(model.state_dict(), str(model_path))
    finally:
        signal.signal(signal.SIGINT, old_handler)

def generate_positions_bin(bin_output_path: Path, config: Config) -> Path:
    chunk_dir = bin_output_path.with_suffix(".chunks")
    # Clean up last iterations positions on start of new iteration
    if chunk_dir.exists():
        shutil.rmtree(chunk_dir)
    if bin_output_path.exists():
        bin_output_path.unlink()
    gen_script = _SCRIPT_DIR / "generate_parallel.sh"
    concat_script = _SCRIPT_DIR / "concat_chunks.sh"
    gen_args = [
        str(gen_script), str(chunk_dir), str(config["workers"]),
        "--num-games", str(config["games_per_worker"]),
        "--max-nodes", str(config["max_nodes"]),
        "--random-half-moves", str(config["random_half_moves"]),
        "--max-half-moves-per-game", str(config["max_half_moves"]),
    ]
    subprocess.run(gen_args, check=True)
    subprocess.run([str(concat_script), str(chunk_dir), str(bin_output_path)], check=True)
    return bin_output_path

def stream_train(config: Config):
    batch_size = config["batch_size"]
    lr = config["lr"]
    model_path = config["save_path"]

    cycles = config["cycles"]
    epochs_per_cycle = config["epochs_per_cycle"]
    val_every = config["val_every"]
    val_path = config["validation_path"]

    device = get_device()
    model = NNUE().to(device)
    if os.path.exists(model_path):
        # weights_only = Non-executable data only, legacy pickler BS  
        model.load_state_dict(torch.load(model_path, map_location=device, weights_only=True))
        print(f"Loaded checkpoint from {model_path}")

    if not os.path.exists(val_path):
        print(f"Validation file {val_path} not found, generating...")
        # TODO IMMEDIATE # of positions for validation file not always the same as for each training round, make it configurable
        generate_positions_bin(Path(val_path), config)

    for cycle in range(1, cycles + 1):
        print(f"\n{'='*60}")
        print(f"CYCLE {cycle}/{cycles}")
        print(f"{'='*60}")

        bin_path = _SCRIPT_DIR / f"_train_cycle_{cycle}.bin"
        print(f"\n[Generate] Producing data...")
        gen_start = time.time()
        generate_positions_bin(bin_path, config)
        gen_elapsed = time.time() - gen_start
        print(f"[Generate] Done in {gen_elapsed:.1f}s")

        print(f"\n[Train] Training for {epochs_per_cycle} epoch(s)...")
        train_start = time.time()
        model = train(
            str(bin_path),
            epochs_per_cycle,
            model,
            batch_size,
            lr,
        )
        train_elapsed = time.time() - train_start
        print(f"[Train] Done in {train_elapsed:.1f}s")

        if cycle % val_every == 0:
            print(f"\n[Validate] Evaluating on {val_path}...")
            val_loss = validate(model, val_path, device)
            print(f"[Validate] Validation loss: {val_loss:.4f}")

        _save_and_rotate(model, Path(model_path), config["backup_count"])

        total_elapsed = gen_elapsed + train_elapsed
        print(f"\n[Summary] Cycle {cycle}: generate={gen_elapsed:.1f}s, train={train_elapsed:.1f}s, total={total_elapsed:.1f}s")

    print("Done")

if __name__ == "__main__":
    stream_train(load_config())
