Basic chess engine compiling to WASM, with browser frontend.

### Basics
- STOP DELETING MY COMMENTS
- See README.md for common commands.
- "Eyeball" tests don't have assertions, you can un-ignore them, run and check the output, but it won't fail.

### Style
- I prefix methods with an underscore if it has a potentially confusing contract, and there is a cleaner (but maybe slower) overload.
- Prefer self-documenting code over pointless comments.
- Prefer dense code with low empty lines.

### Architecture
- When running build, test, or "cargo check", take a CAREFUL look at the commands in README.md and philosophy Cargo.toml. In the end, STOP adding random cfg options.
- Tests are in the same file as the implementation under "mod test".
- Chess engines are highly performance sensitive, and code should reflect this: e.g. bitboard use, cache locality, avoiding branches and copying.
  Look up common chess engine best practices.
