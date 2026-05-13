#!/usr/bin/env bash

# Usage: concat_chunks.sh <input dir> <output file>
#
# Concatenate all *.bin chunks in <input dir> into <output file>, truncating any partial
# trailing entry from each chunk.
# Chunks are not deleted on success - rm them manually.

set -eu

# Must match COMPRESSED_BOARD_SIZE (34) + score (4 bytes) in src/engine/game/board/compressed.rs
ENTRY_SIZE=38

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <input dir> <output file>" >&2
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

# AI: Portable file size (BSD stat on macOS, GNU stat on Linux).
file_size() {
    [ -e "$1" ] || return 1

    stat -f%z "$1" 2>/dev/null ||
    stat -c%s "$1" 2>/dev/null
}

# Lexicographic sort (e.g. 1, 10, 2, 20), not natural order, order doesn't matter so long as it's consistent.
chunks=()
while IFS= read -r line; do  # IFS= preserves spaces in filenames, bash sanity check: IFS= is just PATH= syntax
    chunks+=("$line")
done < <(find "$IN_DIR" -maxdepth 1 -type f -name '*.bin' | sort) # Bash: <() Process substitution <() is a file, yes can input into while loop
#echo DEBUG chunks "${chunks[*]}"

if [ "${#chunks[@]}" -eq 0 ]; then
    echo "No *.bin files in $IN_DIR" >&2
    exit 1
fi

: > "$OUT_FILE" # Creates or clears out file, : does nothing
total_entries=0
truncated_file_count=0
small_removed_count=0
for f in "${chunks[@]}"; do
    size=$(file_size "$f")
    if [ "$size" -lt "$ENTRY_SIZE" ]; then
        echo "WARN: $f is only $size bytes (< $ENTRY_SIZE), removing" >&2
        rm "$f"
        small_removed_count=$((small_removed_count + 1))
        continue
    fi
    entries=$(( size / ENTRY_SIZE )) # Rounds down
    valid_size=$(( entries * ENTRY_SIZE ))
    tail=$(( size - valid_size ))
    if [ "$tail" -ne 0 ]; then
        echo "WARN: $f has $tail trailing bytes, dropping (kept $entries entries)" >&2
        truncated_file_count=$((truncated_file_count + 1))
    fi
    # Append everything into out file
    if [ "$entries" -gt 0 ]; then
        head -c "$valid_size" "$f" >> "$OUT_FILE"
    fi
    total_entries=$((total_entries + entries))
    echo "  $f: $entries entries"
done

out_size=$(file_size "$OUT_FILE")
echo
echo "Wrote $OUT_FILE: $total_entries entries, $out_size bytes from ${#chunks[@]} chunk file(s)."
if [ "$truncated_file_count" -gt 0 ]; then
    echo "Truncated partial tails on $truncated_file_count chunk file(s)."
fi
if [ "$small_removed_count" -gt 0 ]; then
    echo "Removed $small_removed_count file(s) too small to hold even one entry."
fi
