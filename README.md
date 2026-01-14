### A basic chess engine

Rust, WASM, messing around

https://starqi.github.io/ljenks-chess/

#### Deployment

npm run build

Copy www/dist/* to gh-pages branch. 

### TODO

- Saw 2 move draw not 3.

    1. e3 e5
    2. Qg4 d5
    3. Qh5 g6
    4. Qxe5+ Qe7
    5. Bb5+ c6
    6. Qxh8 cxb5
    7. Qxg8 Qf6
    8. Qxh7 Bg4
    9. Nc3 d4
    10. Nxb5 Na6
    11. f3 Bf5
    12. Nxd4 Qb6
    13. Nxf5 gxf5
    14. Qxf5 Nc5
    15. Qe5+ Be7
    16. Qf5 Kf8
    17. Qh7 Ke8
    18. Qf5 Kd8
    19. Qxf7 Kc7
    20. Qxe7+ Kc8
    21. Qf8+ Kc7
    22. Qxa8 Kd7
    23. Qf8 Kc7
    24. Qf7+ Kd8
    25. Qf5 Kc7
    26. Qf8 Kd7
    27. Qf7+ Kc6
    28. Qe8+ Kd6
    29. Qf7 Kc6
    30. Qe8+ Kd6

- No leading move bug when faced with repetition 
- I think pawn push incentive is too low compared to mob, that's why knight keeps checking my king in the end game with +10 pawns
- Saw queen check me to a draw w/o pushing mass amount of pieces -- CONSEQUENCE OF IGNORED MOBILITY WHEN WINNING
- Tests for early game advanced pawns, should gain advantage
    - Position evaluator UI
- Killer move sibling idea is cool
- Flickering when dragging
- Promotion PGN
- AI is thinking graphic, end game screen
- Review wasm bindgen tutorial, ./pkg, not ./node_modules
- Review thread local macro
- / AI scan for major inefficiencies
    - Profiler
- Write notes on how wasm works, check versions
- Minor: Pointer jumps to &Bitboard worse than copying
- ! Choose an opening which forces tactical lines?!
- Pure square control counterexamples: 
    - Long range bishop with all target squares pawn controlled -> not valued
- TODO IMMEDIATE
- Clean up linter warnings
    - Dead code
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
- Take into account attacked piece value
- Count re-search statistics
- If checked, don't do second round of move tests
- Promotion UI
- Faster coarser sort?
- Memo unit tests
- Personal musings - recursive null-move-ish evaluations  
- Abstract away 63 - X, and remove 63 part
- Branchless tricks with repr u8 on data enum
- Draw when kings only
- King safety

Read again
- Debruijin indices
- Wasm Bindgen
- Rust lifetimes, macros

--------------------------------------------------

Usage

- "npm run serve" is enough to compile everything: Rust and JS
    - Doesn't work if not serving from web server
- ? Need syncWebAssembly
