#!/usr/bin/env bash

# Usage: count_positions.sh <input dir>
#
# Count total positions across all *.bin chunks in <input dir>
# by calling chess-cli count on each file. Prints the total.

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CLI="$SCRIPT_DIR/../target/release/chess-cli"
if [ ! -x "$CLI" ]; then
    echo "chess-cli not found at $CLI - build it first, see README" >&2
    exit 2
fi

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <input dir>" >&2
    exit 2
fi

IN_DIR="$1"

if [ ! -d "$IN_DIR" ]; then
    echo "Input directory does not exist: $IN_DIR" >&2
    exit 2
fi

total=0
while IFS= read -r f; do
    n=$("$CLI" count "$f")
    total=$((total + n))
done < <(find "$IN_DIR" -maxdepth 1 -type f -name '*.bin' | sort)

echo "$total"
