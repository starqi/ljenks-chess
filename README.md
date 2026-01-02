### A basic chess engine

Rust, WASM, messing around

https://starqi.github.io/ljenks-chess/

#### Deployment

npm run build
Copy www/dist/* to gh-pages branch. 

### TODO

    1. d3 d6
    2. c4 Bg4
    3. Nc3 Na6
    4. Be3 c5
    5. Qb3 Qd7
    6. O-O-O O-O-O
    7. h3 Be6
    8. Nf3 Nb4
    9. a3 Nc6
    10. Ng5 Nf6
    11. g4 h5
    12. Nxe6 Qxe6
    13. g5 Ne8
    14. Bg2 h4
    15. Bd5 Qd7
    16. Bxf7 Nc7
    17. Nb5 Ne5
    18. g6 Nxf7
    19. gxf7 Nxb5
    20. cxb5 e6
    21. Rdg1 Qxf7
    22. Qa4 Kb8
    23. Rg4 Be7
    24. Qb3 Bf6
    25. Re4 Qh5
    26. Rg1 Qd5
    27. Kc2 Qxb3+
    28. Kxb3 Rde8
    29. Bg5 Rh5
    30. Bxf6 gxf6
    31. Rgg4 Rf5
    32. Rgf4 Rxf4
    33. Rxf4 f5
    34. Rxh4 d5
    35. Ka4 Kc7
    36. Ka5 Rg8
    37. Rh7+ Kc8
    38. Re7 Rg2
    39. Rxe6 Rxf2
    40. Re7 Kd8
    41. Re5 Kc7
    42. Rxd5 Rxe2
    43. Rxc5+ Kd6
    44. Rxf5 Rxb2
    45. Rf8 Ke7
    46. Ra8 Rb3
    47. Ka4 Rxd3
    48. Rxa7 Rxh3
    49. Rxb7+ Kd6
    50. Kb4 Rf3
    51. Rg7 Re3
    52. a4 Re4+
    53. Ka5 Re5
    54. Rf7 Rc5
    55. Ka6 Ke6
    56. Rf4 Ke5
    57. b6 Rc6
    58. Rb4 Kd5
    59. Ka7 Kc5
    60. Rb5+ Kd6
    61. b7 Rc7
    62. Ka8 Kd7
    63. b8 Rc8
    64. Rd5+ Kc6
    65. Rd6+ Kc5
    66. Qxc8+ Kb4
    67. Qb7+ Kc4
    68. Qe4+ Kc3
    69. Rd3+ Kb2
    70. Qg2+ Ka1
    71. Rd1#

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
