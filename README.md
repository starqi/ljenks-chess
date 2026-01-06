### A basic chess engine

Rust, WASM, messing around

https://starqi.github.io/ljenks-chess/

#### Deployment

npm run build
Copy www/dist/* to gh-pages branch. 

### TODO

- PGN so broken:
    
    1. e3 d6
    2. Nf3 g6
    3. Bd3 Bh6
    4. O-O Nf6
    5. b3 O-O
    6. Nc3 Nc6
    7. Ba3 Bf5
    8. Bxf5 gxf5
    9. Nh4 e6
    10. g3 Rb8
    11. f4 b5
    12. Bb2 Bg7
    13. a4 Ne4
    14. axb5 Bxc3
    15. dxc3 Rxb5
    16. c4 Rb4
    17. Qc1 a5
    18. Kg2 Qa8
    19. Kf3 Nc5
    20. Ke2 Ne4
    21. Kf3 Nc5
    22. Rd1 Nxb3
    23. cxb3 f6
    24. Ke2 Rxb3
    25. Kf2 Kf7
    26. Bc3 Qa6
    27. Ra4 Rfb8
    28. Qc2 Nb4
    29. Qc1 c5
    30. Qd2 Qxc4
    31. Bxb4 R7xb4
    32. Rxb4 cxb4
    33. Qxd6 Rb7
    34. Rd4 Qb5
    35. Qd8 Qc5
    36. Qh8 Qc2+
    37. Kf1 Qc1+
    38. Ke2 Qc2+
    39. Rd2 Qxd2+
    40. Kxd2 Rd7+
    41. Kc2 Rc7+
    42. Kb2 Ke7
    43. Qg7+ Kd6
    44. Qxf6 Rc6
    45. Nxf5+ Kc7
    46. Nd4 Ra6
    47. Nxe6+ Rxe6
    48. Qxe6 Kb7
    49. Kb3 Ka7
    50. Ka4 Kb7
    51. Kxa5 Kc7
    52. Kxb4 Kb7
    53. f5 Kc7
    54. f6 Kb7
    55. f7 Kc7
    56. f8 Kb7
    57. Qfc8+ Ka7
    58. Qea6#

- Killer move sibling idea is cool
- Stop running hash move twice
- / Finish sorting issues
- / Memo re-use and aging, check speed... Depth not really re-useable...
- Flickering when dragging
- Promotion PGN
- AI is thinking graphic
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
    - Moving pieces out of the way, similar to synergy
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
