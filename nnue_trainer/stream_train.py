
# TODO REVIEW

import os
import sys
import time
from pathlib import Path

_SCRIPT_DIR = Path(__file__).parent.resolve()
sys.path.insert(0, str(_SCRIPT_DIR))

from generate_parallel import generate_parallel
from trainer import train
from dataset import create_dataloader
import torch
from model import NNUE


def validate(model_path: str, val_path: str, device: torch.device) -> float:
    model = NNUE().to(device)
    model.load_state_dict(torch.load(model_path, map_location=device, weights_only=True))
    model.eval()
    dataloader = create_dataloader(val_path, batch_size=2048, shuffle=False, num_workers=0)
    total_loss = 0
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
    return avg


def stream_train(
    cycles: int = 10,
    games_per_worker: int = 1000,
    epochs_per_cycle: int = 1,
    batch_size: int = 1024,
    lr: float = 1e-3,
    max_nodes: int = 5000,
    random_half_moves: int = 10,
    max_half_moves: int = 200,
    workers: int | None = None,
    model_path: str | None = None,
    validation_path: str | None = None,
    val_every: int = 1,
    cli_path: str | None = None,
):
    model_path = model_path or str(_SCRIPT_DIR / "nnue_model.pt")
    device = torch.device("cpu")

    for cycle in range(1, cycles + 1):
        print(f"\n{'='*60}")
        print(f"CYCLE {cycle}/{cycles}")
        print(f"{'='*60}")

        print(f"\n[Generate] Producing {games_per_worker} games/worker...")
        gen_start = time.time()
        bin_path = generate_parallel(
            games_per_worker=games_per_worker,
            worker_count=workers,
            max_nodes=max_nodes,
            random_half_moves=random_half_moves,
            max_half_moves=max_half_moves,
            cli_path=cli_path,
        )
        gen_elapsed = time.time() - gen_start
        print(f"[Generate] Done in {gen_elapsed:.1f}s")

        print(f"\n[Train] Training for {epochs_per_cycle} epoch(s) on {bin_path}...")
        train_start = time.time()
        train(
            bin_path=bin_path,
            epochs=epochs_per_cycle,
            batch_size=batch_size,
            lr=lr,
            save_path=model_path,
        )
        train_elapsed = time.time() - train_start
        print(f"[Train] Done in {train_elapsed:.1f}s")

        if validation_path and cycle % val_every == 0:
            print(f"\n[Validate] Evaluating on {validation_path}...")
            val_loss = validate(model_path, validation_path, device)
            print(f"[Validate] Validation loss: {val_loss:.4f}")

        os.unlink(bin_path)
        print(f"\n[Cleanup] Deleted {bin_path}")

        total_elapsed = gen_elapsed + train_elapsed
        print(f"\n[Summary] Cycle {cycle}: generate={gen_elapsed:.1f}s, train={train_elapsed:.1f}s, total={total_elapsed:.1f}s")

    print(f"\n{'='*60}")
    print(f"All {cycles} cycles complete. Model saved to {model_path}")
    print(f"{'='*60}")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Continuous generate-train loop for NNUE")
    parser.add_argument("--cycles", type=int, default=10, help="Number of generate-train cycles")
    parser.add_argument("--games-per-worker", type=int, default=1000, help="Games per worker per cycle")
    parser.add_argument("--epochs", type=int, default=1, help="Training epochs per cycle")
    parser.add_argument("--batch-size", type=int, default=1024)
    parser.add_argument("--lr", type=float, default=1e-3)
    parser.add_argument("--max-nodes", type=int, default=5000, help="Search node limit for generation")
    parser.add_argument("--max-half-moves", type=int, default=200, help="Max half moves per game")
    parser.add_argument("--random-half-moves", type=int, default=10)
    parser.add_argument("--workers", type=int, default=None, help="Parallel generation workers")
    parser.add_argument("--model-path", type=str, default=None)
    parser.add_argument("--validation-path", type=str, default=None, help="Fixed validation bin file")
    parser.add_argument("--val-every", type=int, default=1, help="Validate every N cycles")
    parser.add_argument("--cli-path", type=str, default=None)
    args = parser.parse_args()

    stream_train(
        cycles=args.cycles,
        games_per_worker=args.games_per_worker,
        epochs_per_cycle=args.epochs,
        batch_size=args.batch_size,
        lr=args.lr,
        max_nodes=args.max_nodes,
        max_half_moves=args.max_half_moves,
        random_half_moves=args.random_half_moves,
        workers=args.workers,
        model_path=args.model_path,
        validation_path=args.validation_path,
        val_every=args.val_every,
        cli_path=args.cli_path,
    )
