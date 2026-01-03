### A basic chess engine

Rust, WASM, messing around

https://starqi.github.io/ljenks-chess/

#### Deployment

npm run build
Copy www/dist/* to gh-pages branch. 

### TODO

- Extract -(bool) pattern as macro rules
- / Memo re-use and aging, check speed... Depth not really re-useable...
- Flickering when dragging
- Promotion PGN
- AI is thinking graphic
- Review wasm bindgen tutorial, ./pkg, not ./node_modules
- Review thread local macro
- AI scan for major inefficiencies
- Write notes on how wasm works, check versions
- Minor: Pointer jumps to &Bitboard worse than copying
- ! Choose an opening which forces tactical lines?!
- Pure square control counterexamples: 
    - Long range bishop with all target squares pawn controlled -> not valued
- TODO IMMEDIATE
- Profiler?
- Clean up linter warnings
    - Dead code
- Evaluation
    - Moving pieces out of the way, similar to synergy
    - Need to look up by piece, then weigh knight differently b/c less maximum squares attacked
        - Should be able to make queen balanced like this, instead of 0
    - Use a table for square importance
        - Add king area to it
    - Lots of tests
    - Castle bonus should be replaced with king safety?
    - Tapered pawn push eval
    - Pawn structure
    - Piece synergy
- Is CC not fully correct
    - Promotions
    - Currently has if statement, which doesn't count checks...
- Take into account attacked piece value
- Count re-search statistics
- If checked, don't do second round of move tests
- Promotion UI
- Profiler? Necessary to spot any bottlenecks.
- Faster coarser sort?
- 3 fold, and game end draw screen
- Memo unit tests
- Personal musings - recursive null-move-ish evaluations  
- Abstract away 63 - X, and remove 63 part
- Branchless tricks with repr u8 on data enum
- Minor: If start high depth and run out of time, enable at least a random move
    - What is proper iterative deepening? It wastes time.

Read again
- Debruijin indices
- Wasm Bindgen
- Rust lifetimes, macros

--------------------------------------------------

Usage

- "npm run serve" is enough to compile everything: Rust and JS
    - Doesn't work if not serving from web server
- ? Need syncWebAssembly
