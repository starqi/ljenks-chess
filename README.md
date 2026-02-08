### A basic chess engine

Rust, WASM, messing around

https://starqi.github.io/ljenks-chess/

#### Deployment

npm run build

Copy www/dist/* to gh-pages branch. 

##### CLI

Note the project is setup so that default features is WASM for language server,
so in the build command here, need to specify "cli" features and turn off default features,
and cargo check doesn't work either because it scans everything including the WASM bindgen code. 

```bash
cargo build --release --bin chess-cli --no-default-features -F cli
./target/release/chess-cli <FEN> [depth]
```

Evaluate the starting position at depth 10:
```bash
./target/release/chess-cli "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
```

Evaluate a tactical position at depth 12:
```bash
./target/release/chess-cli "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3" 12
```

### TODO

- Review FEN AI code
- FEN - Stop resetting at Rust level if fail to load
- Tailwind?!
- Get rid of ? evaluation and figure out why it loves e6
- TODO IMMEDIATE
- ??? Why mated, clearly broken

    5rk1/ppp4p/2p1p2q/6p1/4P1n1/1PN3Pb/P1PQ1P2/R1B1R1K1 w - - 2 20
    1. e3 e5
    2. Nc3 Nf6
    3. Nf3 Nc6
    4. Bb5 Be7
    5. Bxc6 dxc6
    6. Nxe5 O-O
    7. O-O Bd6
    8. d4 Nd7
    9. Nc4 Bxh2+
    10. Kxh2 Qh4+
    11. Kg1 Nf6
    12. Ne5 Re8
    13. g3 Qh6
    14. b3 Bh3
    15. e4 g5
    16. Re1 Rxe5
    17. dxe5 Ng4
    18. e6 fxe6
    19. Qd2 Rf8
    20. Qd4 Bg2
    21. Kxg2 Qh2+
    22. Kf1 Rxf2+
    23. Qxf2 Qxf2#

- Shit talking based on score drop 
- Solutions for index.js linter errors
- WASM SIMD opportunities
- Killer move sibling idea is cool
- Promotion PGN
- AI is thinking graphic, end game screen
- Review wasm bindgen tutorial, ./pkg, not ./node_modules
- Review thread local macro
- / AI scan for major inefficiencies
    - Profiler
- Write notes on how wasm works, check versions
- Minor: Pointer jumps to &Bitboard worse than copying
- ! Choose an opening which forces tactical lines?!
- * Pure square control counterexamples: 
    - Long range bishop with all target squares pawn controlled -> not valued
- Evaluation
    - Moving pieces out of the way, similar to synergy -> not a problem anymore? Emergent fixed...
    - Need to look up by piece, then weigh knight differently b/c less maximum squares attacked
        - Should be able to make queen balanced like this, instead of 0
    - Use a table for square importance
        - Add king area to it
    - Castle bonus should be replaced with king safety?
    - Tapered pawn push eval
    - Pawn structure
    - Piece synergy?
- Is CC not fully correct
    - Promotions
    - Currently has if statement, which doesn't count checks...
- Count re-search statistics
- If checked, don't do second round of move tests?
- Promotion UI
- Memo unit tests
- ? Personal musings - recursive null-move-ish evaluations  
- Abstract away 63 - X, and remove 63 part
- Branchless tricks with repr u8 on data enum
- Draw when kings only

Minor
- King safety
- Clean up linter warnings
    - Dead code

Read again TODO
- Debruijin indices
- Wasm Bindgen
- lazy_static macros
- Rust lifetimes, '_
- Rust split() collect() magic
- Rust parse() magic

--------------------------------------------------

Usage

- "npm run serve" is enough to compile everything: Rust and JS
    - Doesn't work if not serving from web server
- ? Need syncWebAssembly


### FENs

- 2q1k3/8/8/8/8/8/8/4K3 w - - 0 1
- k7/6Q1/7R/8/8/4q3/8/4K3 w - - 0 1
