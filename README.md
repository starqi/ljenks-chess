TODO

- Tests for evaluation
/ Bitboards
- If checked, don't do second round of move tests
/ Promotion UI
- En passant
- Profiler? Curious.

--------------------------------------------------

- Proper transposition table - stop clearing it
- Can replace aggression eval with handler as well, saving move allocation
- Promotions and checks should be marked - not quiet!
- UI: Disallow fake premoves
- En passant + old board state
- Investigate Webpack Wasm generation
- Debug build, put logs inside debug
- Split off quiescence, less branches
- Lazy evaluation with incremental material heuristic, maybe also with control squares?
- Personal musings - recursive null-move-ish evaluations  

--------------------------------------------------

Usage

- "npm run serve" is enough to compile everything: Rust and JS
    - Doesn't work without if not serving from web server
- Need syncWebAssembly
