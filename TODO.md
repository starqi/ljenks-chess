
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

- Does NNUE game gen END? Why or why not?
- Basic overfitting test
- ??? AI anecdote on ability to understand NNUE 
- TODO IMMEDIATE
    - NNUE related ones
- // TODO temp_moves unnecessary? Try it
- / Review new main.rs AI code
- Get rid of ? evaluation and figure out why it loves e6
- // TODO Refactor into struct
- Performance bottlenecks review
- Bitbucket & other places backup
- // TODO How does this work as part of build process?
- Solutions for index.js linter errors
- WASM SIMD opportunities
- Killer move sibling idea is very cool
- Promotion PGN
- Review wasm bindgen tutorial, ./pkg, not ./node_modules
- / AI scan for major inefficiencies
    - Profiler
- Write notes on how wasm works, check versions
- Perf: Pointer jumps to &Bitboard worse than copying?
- Evaluation
    - Moving pieces out of the way, similar to synergy -> not a problem anymore? Emergent fixed...

    - Fixed already? Need to look up by piece, then weigh knight differently b/c less maximum squares attacked
        - Should be able to make queen balanced like this, instead of 0
    - Castle bonus should be replaced with king safety?
    - Piece synergy?
- Is CC not fully correct
    - Promotions
    - Currently has if statement, which doesn't count checks...
- Count re-search statistics
- !? If checked, don't do second round of move tests?
- Finish promotions and UI
- Branchless tricks with repr u8 on data enum
- Draw when kings only still not done
- Metadata for Discord

Shower thoughts
- Choose an opening which forces tactical lines?!
- Write comment? Pure square control counterexamples: 
    - Long range bishop with all target squares pawn controlled -> not valued
- Gauge potential for Playwright automation
- Shit talking based on score drop 
- // TODO (Feature Req) Outside ability to set the board to mostly anything without breaking hash,
// right now tests have responsibilty to maintain proper state

Minor
- AI is thinking graphic
- FEN - Stop resetting the board if fail to load
- // Enabling ANY of these slowed down NPS by 4x, why? TODO (Minor)
- King safety
- Clean up linter warnings
    - Dead code
- Unused vars Rust warnings
- Memo unit tests
- Upgrade Rust edition to 2024, actually fails

Read again
- Debruijin indices
- Wasm Bindgen
- lazy_static macros
- Rust lifetimes, '_
- Rust split() collect() magic
- Rust parse() magic
- Rust from/into
- Thread local macro
- derive Parser, clap
- dyn
- Iterable Iter and string Split wrappers
- Autograd

--------------------------------------------------

### FENs

- 2q1k3/8/8/8/8/8/8/4K3 w - - 0 1
- k7/6Q1/7R/8/8/4q3/8/4K3 w - - 0 1
- 5rk1/ppp4p/2p1p2q/6p1/4P1n1/1PN3Pb/P1PQ1P2/R1B1R1K1 w - - 2 20
    - Win without getting mated in volatile position, works when given enough depth
