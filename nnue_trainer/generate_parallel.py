import subprocess
import tempfile
import os
import argparse
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

_SCRIPT_DIR = Path(__file__).parent
_DEFAULT_CLI = Path(__file__).parent.parent / "target" / "release" / "chess-cli"


def _generate_chunk(
    worker_id: int,
    cli_path: str,
    num_games: int,
    max_nodes: int,
    random_half_moves: int,
    max_half_moves: int,
) -> tuple[int, str, int]:
    # TODO IMMEDIATE Robustness strategy? User cancels, unknown subprocess failure,
    # where is the temp file? Just lying around on my system? I don't want to lose all my preciously generated positions.
    # How is this resumable? Design resume strategy such that 
    # if I pick abc.bin as the final file name, maybe this actually makes a folder 
    # instead and tracks the expected total # of final positions in some key/value config file.
    # Then upon any kind of failure scenario, I can resume generating abc.bin.
    # And if abc.bin is complete already, then do nothing. 
    tmp = tempfile.NamedTemporaryFile(suffix=".bin", delete=False, dir=_SCRIPT_DIR)
    tmp_path = tmp.name
    tmp.close()
    try:
        cmd = [
            cli_path, "generate", tmp_path,
            "--num-games", str(num_games),
            "--max-nodes", str(max_nodes),
            "--random-half-moves", str(random_half_moves),
            "--max-half-moves-per-game", str(max_half_moves),
            "--quiet", # TODO IMMEDIATE Check that --quiet should still report progress? Won't I get a blank frozen CLI? How do I display progress in a multi-process worker setup?
        ]

        start = time.time()
        result = subprocess.run(cmd, capture_output=True, text=True)
        elapsed = time.time() - start

        if result.returncode != 0:
            raise RuntimeError(f"Worker {worker_id} failed: {result.stderr}")

        file_size = os.path.getsize(tmp_path)
        positions = file_size // 38 # TODO IMMEDIATE Use constant, add comment referencing source of truth compressed.rs
        print(f"  Worker {worker_id}: {positions} positions in {elapsed:.1f}s ({positions / elapsed:.0f} pos/s)")
        return worker_id, tmp_path, positions
    except:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)
        raise


def generate_parallel(
    games_per_worker: int = 1000,
    worker_count: int | None = None,
    max_nodes: int = 5000,
    random_half_moves: int = 10,
    max_half_moves: int = 200,
    cli_path: str | None = None,
    output_path: str | None = None,
) -> str:
    cli_path = cli_path or str(_DEFAULT_CLI)
    if not os.path.exists(cli_path):
        raise FileNotFoundError(f"CLI not found at {cli_path}")

    worker_count = worker_count or min(os.cpu_count() or 4, 10)
    start = time.time()
    results: list[tuple[int, str, int]] = [] # TODO IMMEDIATE Named tuples

    with ProcessPoolExecutor(max_workers=worker_count) as pool: # TODO IMMEDIATE (Read)
        futures = []
        for i in range(worker_count):
            futures.append(pool.submit(
                _generate_chunk,
                i,
                cli_path,
                games_per_worker,
                max_nodes,
                random_half_moves,
                max_half_moves))
        for future in as_completed(futures):
            results.append(future.result())

    results.sort(key=lambda x: x[0])
    total_positions = sum(r[2] for r in results)
    elapsed = time.time() - start

    # Concat temp files together
    output_path = output_path or str(_SCRIPT_DIR / "generated.bin")
    with open(output_path, "wb") as out_f:
        for _, tmp_path, _ in results:
            with open(tmp_path, "rb") as in_f:
                out_f.write(in_f.read())
            os.unlink(tmp_path)

    print(f"\nGenerated {total_positions} positions in {elapsed:.1f}s ({total_positions / elapsed:.0f} pos/s)")
    print(f"Output: {output_path}")
    return output_path


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate training positions in parallel")
    parser.add_argument("--games-per-worker", type=int, default=100, help="Number of games per worker")
    parser.add_argument("--workers", type=int, default=None, help="Number of parallel workers (default: CPU count, max 10)")
    parser.add_argument("--max-nodes", type=int, default=5000, help="Search node limit per move")
    # TODO IMMEDIATE Won't this be stagnant beyond the initial randomness? 
    parser.add_argument("--random-half-moves", type=int, default=16, help="Random moves at start of each game")
    parser.add_argument("--max-half-moves", type=int, default=200, help="Max half moves per game")
    # TODO IMMEDIATE Why can't ../.. be default string? Why do this progammatically?
    parser.add_argument("--cli-path", type=str, default=None, help="Path to CLI binary for this project")
    # TODO IMMEDIATE Bin file must be required, no "generated.bin", needs to provide a name as an identitifer to know when to RESUME the same bin
    parser.add_argument("--output", type=str, default=None, help="Output bin file path")
    args = parser.parse_args()

    generate_parallel(
        games_per_worker=args.games_per_worker,
        worker_count=args.workers,
        max_nodes=args.max_nodes,
        random_half_moves=args.random_half_moves,
        max_half_moves=args.max_half_moves,
        cli_path=args.cli_path,
        output_path=args.output,
    )
