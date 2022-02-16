TODO

- Evaluation
    . Castle bonus, currently at 0!
    - Moving pieces out of the way, similar to synergy
    - Need to look up by piece, then weigh knight differently b/c less maximum squares attacked
        - Should be able to make queen balanced like this, instead of 0
    - Use a table for square importance
        - Add king area to it
    - Lots of tests
    . Pawn push but only in end game
    - Pawn structure
    - Piece synergy
- Is CC not fully correct
    - Promotions
    - Currently has if statement, which doesn't count checks...
. What about attacking a piece? Not part of mobility score.
. Missed opponent side mobility eval
- Save best alpha to memo if terminated
. Stop cutting remaining depth if <= 3 in LMR...
	. Then add search extensions
- If checked, don't do second round of move tests
- Promotion UI
- Profiler? Necessary to spot any bottlenecks.
- Faster coarser sort
- Prune memo
- Memo unit tests
- Proper transposition table - stop clearing it
- UI: Disallow fake premoves
- En passant + old board state
- Investigate Webpack Wasm generation
- Debug build, put logs inside debug
- Personal musings - recursive null-move-ish evaluations  
- Abstract away 63 - X, and remove 63 part
- Branchless tricks with repr u8 on data enum

--------------------------------------------------

Usage

- "npm run serve" is enough to compile everything: Rust and JS
    - Doesn't work without if not serving from web server
- Need syncWebAssembly
