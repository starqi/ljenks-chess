### A basic chess engine

https://starqi.github.io/ljenks-chess/

Rust, WASM, messing around.

Rough goal:
- Simple obvious hand evaluations -> emergent ability through search and NNUE.
- Add some entertainment features. 
- Don't care about formal UCI compliance.

#### WASM

```bash
# Deploy to gh-pages
npm run build
# Copy www/dist/* to gh-pages branch. 

# Local testing
cd www
npm run serve # Enough to compile everything: Rust and JS, doesn't work if not serving from web server

```

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

- What the fuck?

    1. d4 e6
    2. e4 Qh4
    3. Nc3 Bb4
    4. Nf3 Qxe4+
    5. Be2 Bxc3+
    6. bxc3 Ne7
    7. O-O O-O
    8. Bd3 Qc6
    9. Ng5 Qxc3
    10. Bxh7+ Kh8
    11. Qh5 Qd3
    12. Bg6+ Kg8
    13. Qh7#

- REGRESSION? Auto play when dragging enemy piece
- Review FEN AI code
- FEN - Stop resetting at Rust level if fail to load
- Tailwind?!
- Get rid of ? evaluation and figure out why it loves e6
- TODO IMMEDIATE
- Performance bottlenecks review
- // TODO How does this work as part of build process?
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
- // Enabling ANY of these slowed down NPS by 4x, why? TODO (Minor)
- King safety
- Clean up linter warnings
    - Dead code

Read again
- Debruijin indices
- Wasm Bindgen
- lazy_static macros
- Rust lifetimes, '_
- Rust split() collect() magic
- Rust parse() magic
- Rust from/into

--------------------------------------------------

### FENs

- 2q1k3/8/8/8/8/8/8/4K3 w - - 0 1
- k7/6Q1/7R/8/8/4q3/8/4K3 w - - 0 1
- 5rk1/ppp4p/2p1p2q/6p1/4P1n1/1PN3Pb/P1PQ1P2/R1B1R1K1 w - - 2 20
    - Win without getting mated in volatile position, works when given enough depth
