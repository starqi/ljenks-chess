TODO

- Use memo for 0 depth eval...
- Fix quiescence BS
    - Why quiescence low NPS
    - Where's the eval
    - Intuition
    - Stop computing non-captures
    - Split off, no branches
- Evaluation
    - Need to look up by piece, then weigh knight differently b/c less maximum squares attacked
        - Should be able to make queen balanced like this, instead of 0
    - Use a table for square importance
        - Add king area to it
    - Lots of tests
    - Not done but now looks necessary: Pawn push but only in end game, pawn structure
- Tests for evaluation
- LMR works but move ordering is probably bad
- If checked, don't do second round of move tests
/ Promotion UI
- En passant
- Profiler? Necessary to spot any bottlenecks.
- Faster coarser sort
- Prune memo
- Memo unit tests
- Proper transposition table - stop clearing it
- Can replace aggression eval with handler as well, saving move allocation
- Promotions and checks should be marked - not quiet!
- UI: Disallow fake premoves
- En passant + old board state
- Investigate Webpack Wasm generation
- Debug build, put logs inside debug
- Lazy evaluation with incremental material heuristic, maybe also with control squares?
- Personal musings - recursive null-move-ish evaluations  
- Abstract away 63 - X, and remove 63 part
- Branchless tricks with repr u8 on data enum

--------------------------------------------------

Usage

- "npm run serve" is enough to compile everything: Rust and JS
    - Doesn't work without if not serving from web server
- Need syncWebAssembly
