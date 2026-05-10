#!/usr/bin/env bash
# Concatenate all *.bin chunks in IN_DIR into OUT_FILE, truncating any partial
# trailing entry from each chunk (a chunk killed mid-write may have <38 dangling
# bytes at the tail). Chunks are NOT deleted on success - rm them yourself once
# you've verified OUT_FILE.
#
# Usage: concat_chunks.sh IN_DIR OUT_FILE

# TODO IMMEDIATE Review 

set -eu

# Must match COMPRESSED_BOARD_SIZE (34) + score (4 bytes) in src/engine/game/board/compressed.rs
ENTRY_SIZE=38

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 IN_DIR OUT_FILE" >&2
    exit 2
fi

IN_DIR="$1"
OUT_FILE="$2"

if [ ! -d "$IN_DIR" ]; then
    echo "Input directory does not exist: $IN_DIR" >&2
    exit 2
fi
if [ -e "$OUT_FILE" ]; then
    echo "Output file already exists: $OUT_FILE (delete or pick another path)" >&2
    exit 2
fi

# Portable file size (BSD stat on macOS, GNU stat on Linux).
file_size() {
    stat -f%z "$1" 2>/dev/null || stat -c%s "$1"
}

# Sort numerically by basename when names are integers, otherwise lexicographic.
# Using read loop instead of mapfile for bash 3.2 (macOS default) compatibility.
chunks=()
while IFS= read -r line; do
    chunks+=("$line")
done < <(find "$IN_DIR" -maxdepth 1 -type f -name '*.bin' | sort -V)

if [ "${#chunks[@]}" -eq 0 ]; then
    echo "No *.bin files in $IN_DIR" >&2
    exit 1
fi

: > "$OUT_FILE"
total_entries=0
truncated_files=0
for f in "${chunks[@]}"; do
    size=$(file_size "$f")
    entries=$(( size / ENTRY_SIZE ))
    valid=$(( entries * ENTRY_SIZE ))
    tail=$(( size - valid ))
    if [ "$tail" -ne 0 ]; then
        echo "WARN: $f has $tail trailing bytes, dropping (kept $entries entries)" >&2
        truncated_files=$((truncated_files + 1))
    fi
    if [ "$entries" -gt 0 ]; then
        # head -c is byte-precise on both BSD (macOS) and GNU and avoids dd's
        # oflag=append portability issue. Append via >> to OUT_FILE.
        head -c "$valid" "$f" >> "$OUT_FILE"
    fi
    total_entries=$((total_entries + entries))
    echo "  $f: $entries entries"
done

out_size=$(file_size "$OUT_FILE")
echo
echo "Wrote $OUT_FILE: $total_entries entries, $out_size bytes from ${#chunks[@]} chunk file(s)."
if [ "$truncated_files" -gt 0 ]; then
    echo "Truncated partial tails on $truncated_files chunk file(s)."
fi
