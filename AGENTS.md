Basic chess engine compiling to WASM, with browser frontend.

### Basics
- STOP DELETING MY COMMENTS
- See README.md for common commands.
- "Eyeball" tests don't have assertions, you can un-ignore them, run and check the output, but it won't fail.

### Style notes
- I prefix methods with an underscore if it has a potentially confusing contract, and there is a cleaner (but maybe slower) overload.
- Prefer self-documenting code over pointless comments.
- Prefer dense code with low empty lines.

### Architecture notes
- When running build, test, or "cargo check", take a careful look at the commands in README.md and philosophy Cargo.toml.
    - Stop adding random cfg options.
- Tests are in the same file as the implementation under "mod test".
- Chess engines are highly performance sensitive, and code should reflect this: e.g. bitboard use, cache locality, avoiding branches and copying.
  Look up common chess engine best practices.

### Navigation map

NNUE is INCOMPLETE (TODO).

```
src/
  lib.rs                         - WASM interface from JS, also shared by main.rs the CLI tool for NNUE
  macros.rs                      
  main.rs                        - CLI binary
  engine/
    mod.rs
    ai/                          - By AI, I mean search
      evaluation.rs              - Counting material, mobility, etc.
      move_buckets.rs            - Helper
      mod.rs                     
    game/
      bitboard.rs                - Bitboard manipulation
      bitboard_presets.rs        - Precomputed bitboard constants (files, ranks, etc)
      board/                     - A Chess board in memory
        mod.rs
        moves.rs                 - Layer above Bitboard move gen
        fen.rs                   - FEN parsing
        stringify.rs             - Move-to-notation strings
        nnue.rs                  - NNUE helpers
        compressed.rs            - Compressed board representation for exporting to NNUE
      castle_utils.rs            
      coords.rs                  
      entities.rs                - Piece, Player, Square, etc
      memo.rs                    - Transposition / hash table / "memo"
      move_list.rs               
      move_gen.rs                - Bitboard move gen               
      searchable_moves.rs        - Legal move lookup for UI (given from/to coords)
  platform/                      - Selects between implementations for random and logging (very different on browser vs native)
    mod.rs                       
    wasm.rs                      
    cli.rs                       
www/                             - Web browser frontend
  worker_shared.js
  worker_nnue.js
  worker_no_nnue.js
  index.js                       - Main UI
  index.html / styles.css
nnue_trainer/                    - Python NNUE trainer (in progress)
```
