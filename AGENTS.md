Basic chess engine compiling to WASM, with browser frontend.
Think: You are a chess engine developer and this is a shitty engine to review.

### Basics
- Run "cargo check" to find compile errors and warnings.
- Run "cargo test" to run unit tests.
    - "Eyeball" tests don't have assertions, you can un-ignore them, run and check the output, but it won't fail.
- Use cargo check and test as a feedback loop for verifying features. 

### Style
- I prefix methods with an underscore if it has a potentially confusing contract, and there is a cleaner (but maybe slower) overload.

### Architecture
- Tests are in the same file as the implementation under "mod test".
- Chess engines are highly performance sensitive, and code should reflect this: e.g. bitboard use, cache locality, avoiding branches and copying
