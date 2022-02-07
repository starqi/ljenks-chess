TODO

- Tests for evaluation
- Evaluation using bitboards
/ Bitboards
- If checked, don't do second round of move tests
/ Promotion UI
- En passant
- Profiler? Necessary to spot any bottlenecks.
- Fix quiescence BS
    - Why quiescence low NPS
    - Where's the eval
    - Intuition
    - Stop computing non-captures
    - Split off, no branches
- Faster coarser sort
- Prune memo
- King pos still broken in certain cases

--------------------------------------------------

- Proper transposition table - stop clearing it
- Can replace aggression eval with handler as well, saving move allocation
- Promotions and checks should be marked - not quiet!
- UI: Disallow fake premoves
- En passant + old board state
- Investigate Webpack Wasm generation
- Debug build, put logs inside debug
- Lazy evaluation with incremental material heuristic, maybe also with control squares?
- Personal musings - recursive null-move-ish evaluations  

--------------------------------------------------

Usage

- "npm run serve" is enough to compile everything: Rust and JS
    - Doesn't work without if not serving from web server
- Need syncWebAssembly
