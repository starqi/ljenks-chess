#!/usr/bin/env bash

# Spawn N parallel chess-cli generate subprocesses, each writing to its own
# NEW numbered .bin file inside OUT_DIR. Re-run after Ctrl-C / crash and new files
# are added alongside survivors (numbering continues from max+1). Concat later
# with concat_chunks.sh.

set -u
# Random bash & AI gen note: a lot of "<command> || true" is only needed when "set -e" is enabled

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <output dir> <number parallel processes> [chess-cli generate args...]" >&2
    exit 2
fi

OUT_DIR="$1"
NPROC="$2"
shift 2

# Strip --quiet/-q from user args; we always force --quiet to avoid interleaved
# chess boards from N processes. Clap rejects duplicate --quiet flags.
adjusted_args=()
for arg in "$@"; do
    case "$arg" in 
        --quiet|-q) ;;
        *) adjusted_args+=("$arg") ;;
    esac
done

if ! [[ "$NPROC" =~ ^[0-9]+$ ]] || [ "$NPROC" -lt 1 ]; then
    echo "NPROC must be a positive integer, got: $NPROC" >&2
    exit 2
fi

# Get abs path in case $0 involves .. or . and note nested quotations inside $() is fine
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# HARD CODING relative Rust/Python folder structure
CLI="$SCRIPT_DIR/../target/release/chess-cli"
if [ ! -x "$CLI" ]; then
    echo "chess-cli not found at $CLI - build it first, see README" >&2
    exit 2
fi

mkdir -p "$OUT_DIR"

# Find max existing integer-named *.bin
max=0
for f in "$OUT_DIR"/*.bin; do
    [ -e "$f" ] || continue # Safeguard
    name="$(basename "$f" .bin)" # Deletes front of path, trailing slash, and .bin suffix if it has .bin suffix
    if [[ "$name" =~ ^[0-9]+$ ]] && [ "$name" -gt "$max" ]; then
        max="$name"
    fi
done

# New parallel process IDs, new bin files
pids=()
files=()

# Install trap BEFORE spawning so children inherit it correctly and we never
# have a window where signals are ignored.
cleanup() {
    # Term workers then ignore the SIGINT/TERM, script will end when the workers end
    echo
    echo "Sig INT or TERM received, forwarding SIGTERM to ${#pids[@]} workers..." >&2
    # AI: The mechanism which prevents Ctrl-C / SIGINT from killing child workers (mapping it to SIG_IGN)
    # also prevents SIGINT propagation here from working, so use SIGTERM. 
    if [ "${#pids[@]}" -gt 0 ]; then
        kill -TERM "${pids[@]}" 2>/dev/null
    fi
}
trap cleanup INT TERM

for i in $(seq 1 "$NPROC"); do
    n=$((max + i))
    out="$OUT_DIR/$n.bin"
    files+=("$out")
    # Force --quiet (stripped from user args above)
    "$CLI" generate "$out" "${adjusted_args[@]}" --quiet &
    pid=$!
    pids+=("$pid")
    echo "Spawned worker $i -> $out (pid $pid)"
done

# Wait for child processes via kill -0 (checking if pid exists still) and try to gather status codes.

# Adversarial test example: Consider if all children unresponsive, then we would be ctrl-C'ing repeatedly 
# sending pointless signals and this keeps waiting. 

for pid in "${pids[@]}"; do
    rc=1337
    while kill -0 "$pid" 2>/dev/null; do
        wait "$pid" 2>/dev/null
        rc=$?
    done
    if [ "$rc" -eq 1337 ] || [ "$rc" -gt 128 ]; then # If no real code exists, try to get it. See convention for 128 + sig code.
        wait "$pid" 2>/dev/null
        rc=$?
    fi
    echo "Worker code" $rc
done

# Resets the trap from "cleanup" assigned above
trap - INT TERM

echo
echo "Workers done."
echo "Chunk files in $OUT_DIR:"
ls -lh "${files[@]}" 2>/dev/null
