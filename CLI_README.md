# Chess Engine CLI

A native command-line tool for evaluating chess positions using the ljenks-chess engine.

## Building

```bash
cargo build --release --bin chess-cli
```

The binary will be created at `target/release/chess-cli`.

## Usage

```bash
./target/release/chess-cli <FEN> [depth]
```

### Arguments

- `FEN`: FEN string representing the chess position
- `depth`: Search depth (default: 10)

### Examples

Evaluate the starting position at depth 10:
```bash
./target/release/chess-cli "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
```

Evaluate a tactical position at depth 12:
```bash
./target/release/chess-cli "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3" 12
```

## Output

The tool outputs:
- Evaluation score (positive = white advantage, negative = black advantage)
- Number of nodes searched
- Time taken
- Nodes per second (NPS)
- Best move found

### Example Output

```
Evaluating position at depth 10...
FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

Evaluation: 2
Nodes searched: 22916
Time: 0.05s
NPS: 456858
Best move: P e3 ordering=2, metadata=0
```

## Notes

- The evaluation score is from the perspective of the player to move
- Higher depth = more accurate but slower evaluation
- The engine uses alpha-beta pruning with iterative deepening
- Transposition table (memoization) is used for performance