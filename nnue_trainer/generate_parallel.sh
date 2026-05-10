#!/usr/bin/env bash
# Spawn N parallel chess-cli generate subprocesses, each writing to its own
# numbered .bin file inside OUT_DIR. Re-run after Ctrl-C / crash and new files
# are added alongside survivors (numbering continues from max+1). Concat later
# with concat_chunks.sh.
#
# Usage: generate_parallel.sh OUT_DIR NPROC [forwarded chess-cli generate args...]
# Example:
#   nnue_trainer/generate_parallel.sh nnue_trainer/chunks 10 \
#     --num-games 1000 --max-nodes 100000 --quiet

# TODO IMMEDIATE Review 
# Force --quiet?

set -u

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 OUT_DIR NPROC [chess-cli generate args...]" >&2
    exit 2
fi

OUT_DIR="$1"
NPROC="$2"
shift 2

if ! [[ "$NPROC" =~ ^[0-9]+$ ]] || [ "$NPROC" -lt 1 ]; then
    echo "NPROC must be a positive integer, got: $NPROC" >&2
    exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CLI="$SCRIPT_DIR/../target/release/chess-cli"
if [ ! -x "$CLI" ]; then
    echo "chess-cli not found at $CLI - build it first:" >&2
    echo "  cargo build --release --bin chess-cli --no-default-features -F cli" >&2
    exit 2
fi

mkdir -p "$OUT_DIR"

# Find max existing integer-named *.bin so re-runs don't collide.
max=0
for f in "$OUT_DIR"/*.bin; do
    [ -e "$f" ] || continue
    name="$(basename "$f" .bin)"
    if [[ "$name" =~ ^[0-9]+$ ]] && [ "$name" -gt "$max" ]; then
        max="$name"
    fi
done

pids=()
files=()

# Install trap BEFORE spawning so children inherit it correctly and we never
# have a window where signals are ignored. Forwards Ctrl-C / SIGTERM to
# children so they can flush. Concat truncates any partial tails.
cleanup() {
    echo
    echo "Signal received, forwarding SIGTERM to ${#pids[@]} workers..." >&2
    # SIGTERM (not SIGINT) because bash sets SIGINT to SIG_IGN for background
    # children of interactive shells. SIGTERM gets through reliably.
    if [ "${#pids[@]}" -gt 0 ]; then
        kill -TERM "${pids[@]}" 2>/dev/null || true
    fi
}
trap cleanup INT TERM

for i in $(seq 1 "$NPROC"); do
    n=$((max + i))
    out="$OUT_DIR/$n.bin"
    files+=("$out")
    "$CLI" generate "$out" "$@" &
    pid=$!
    pids+=("$pid")
    echo "Spawned worker $i -> $out (pid $pid)"
done

# Wait for each child individually. If a trap interrupts wait, bash returns
# >128 and we re-check whether the child is still running, looping until the
# child is reaped or already gone. Then move to the next pid.
fail=0
for pid in "${pids[@]}"; do
    while kill -0 "$pid" 2>/dev/null; do
        wait "$pid" 2>/dev/null
        rc=$?
        # rc >128 means wait was interrupted by a signal (our trap fired).
        # Loop back and re-wait. Otherwise the child has been reaped.
        if [ "$rc" -le 128 ]; then break; fi
    done
    # Final wait to harvest exit code (returns immediately, child is reaped).
    wait "$pid" 2>/dev/null
    rc=$?
    # rc 127 = unknown pid (already harvested). Treat 0 and 127 as success.
    if [ "$rc" -ne 0 ] && [ "$rc" -ne 127 ]; then
        fail=$((fail + 1))
    fi
done

trap - INT TERM

echo
echo "Workers done. Failures: $fail / $NPROC"
echo "Chunk files in $OUT_DIR:"
ls -lh "${files[@]}" 2>/dev/null || true

if [ "$fail" -gt 0 ]; then
    exit 1
fi
