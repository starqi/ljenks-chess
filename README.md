### A basic chess engine

https://starqi.github.io/ljenks-chess/

Rust, WASM, messing around.

Rough goal:
- Simple obvious hand evaluations -> emergent ability through search and NNUE.
- Add some entertainment features. 
- Don't care about formal UCI compliance for now.

#### Tests, compile check

These don't require "cli" or "wasm" features, and just scan/test everything, see Cargo.toml philosophy.
```bash
cargo check
cargo test
cargo test my_test_name -- --nocapture
```

#### WASM

```bash

# Deploy to gh-pages
npm run build # Both JS and Rust
# Copy www/dist/* to gh-pages branch. 

# Local testing
cd www
npm run serve # Enough to compile everything: btoh Rust and JS. Doesn't work if not serving from web server (open up HTML file).
```

#### CLI (For NNUE, not used as a normal engine)

```bash
cargo build --release --bin chess-cli --no-default-features -F cli

# Generate positions into output file, see options
./target/release/chess-cli <binary output filename>
./target/release/chess-cli --view 5-10 <binary output filename>
```

#### Python trainer 

All paths configured in `nnue_trainer/configs/default.yaml`.
```bash 
cd nnue_trainer
pip install -r requirements.txt
# TODO .venv

# Generate training data (see CLI section)
# Copy the .bin file path into configs/default.yaml bin_path

# Paths are in configs/
python train.py
python check_model.py
python export_safetensors.py
```
