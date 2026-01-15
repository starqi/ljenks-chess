Basic chess engine compiling to WASM, with browser frontend.

### Basics
- Run "cargo check" to find compile errors and warnings.
- Run "cargo test" to run unit tests.
    - "Eyeball" tests don't have assertions, you can un-ignore them, run and check the output, but it won't fail.
- Use cargo check and test as a feedback loop for verifying features. 

### Style
- I prefix methods with an underscore if it has a potentially confusing contract, and there is a cleaner (but maybe slower) overload.
- Prefer self-documenting code over pointless comments.

### Architecture
- Tests are in the same file as the implementation under "mod test".
- Chess engines are highly performance sensitive, and code should reflect this: e.g. bitboard use, cache locality, avoiding branches and copying.
  Use your expert chess engine skills to analyze this crappy version.
