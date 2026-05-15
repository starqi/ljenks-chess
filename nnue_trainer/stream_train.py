import os
import shutil
import subprocess
import time
import traceback
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

def generate_positions_bin(bin_output_path: Path, config: Config, chunk_dir: Path | None = None) -> Path:
    if chunk_dir is None:
        chunk_dir = bin_output_path.with_suffix(".chunks")
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
    shutil.rmtree(chunk_dir, ignore_errors=True) # TODO IMMEDIATE Read, and what happens if chunk dir not removed? Don't want old files in there! Need to throw.
    # TODO IMMEDIATE Just check that everything in this method throws if something is wrong 
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

        try:
            bin_path = _SCRIPT_DIR / f"_train_cycle_{cycle}.bin"
            print(f"\n[Generate] Producing data...")
            gen_start = time.time()
            generate_positions_bin(bin_path, config)
            gen_elapsed = time.time() - gen_start
            print(f"[Generate] Done in {gen_elapsed:.1f}s")

            print(f"\n[Train] Training for {epochs_per_cycle} epoch(s)...")
            train_start = time.time()
            model = train(
                bin_path=str(bin_path),
                epochs=epochs_per_cycle,
                batch_size=batch_size,
                lr=lr,
                save_path=model_path,
                model=model,
                device=device,
            )
            train_elapsed = time.time() - train_start
            print(f"[Train] Done in {train_elapsed:.1f}s")

            if cycle % val_every == 0:
                print(f"\n[Validate] Evaluating on {val_path}...")
                val_loss = validate(model, val_path, device)
                print(f"[Validate] Validation loss: {val_loss:.4f}")

            # TODO IMMEDIATE If this fails, does it raise and catch below?
            os.unlink(bin_path)

            total_elapsed = gen_elapsed + train_elapsed
            print(f"\n[Summary] Cycle {cycle}: generate={gen_elapsed:.1f}s, train={train_elapsed:.1f}s, total={total_elapsed:.1f}s")
        except Exception:
            # TODO IMMEDIATE What should we do when exception is caught? I'm thinking 
            # model needs to be saved every cycle (below comment) and this this should just end the training.
            # And human can resume from the saved model.
            print(f"\n[ERROR] Cycle {cycle} failed:")
            traceback.print_exc() # Print latest exception info
            # TODO IMMEDIATE Reuse above path
            cleanup = _SCRIPT_DIR / f"_train_cycle_{cycle}.bin"
            if cleanup.exists():
                cleanup.unlink(missing_ok=True)
            chunk_dir = cleanup.with_suffix(".chunks")
            if chunk_dir.exists():
                shutil.rmtree(chunk_dir, ignore_errors=True)
            print("[INFO] Continuing to next cycle...")

    # TODO SAVE EVERY CYCLE
    torch.save(model.state_dict(), model_path)
    print("Done")

if __name__ == "__main__":
    stream_train(load_config())
