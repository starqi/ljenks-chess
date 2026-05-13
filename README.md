### A basic chess engine

https://starqi.github.io/ljenks-chess/

Rust, WASM, messing around. 
NNUE INCOMPLETE.

Rough goal:
- Simple obvious hand evaluations -> emergent ability through search and self-bootstrapped NNUE.
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

# Generate and APPEND positions into output file (see --help for options)
./target/release/chess-cli generate nnue_trainer/xyz.bin

# Parallel: spawn N processes, each writes its own NEW numbered .bin into an output directory
nnue_trainer/generate_parallel.sh nnue_trainer/chunks 10 --num-games 1 --max-nodes 30000

# Concat all *.bin in output directory into one file and fixes partial trailing entries due to interrupt or error
# Does not delete chunks; rm them manually
nnue_trainer/concat_chunks.sh nnue_trainer/chunks nnue_trainer/positions.bin

# View positions/score (training data) from a .bin file
./target/release/chess-cli view xyz.bin 5
./target/release/chess-cli view xyz.bin 1-10
./target/release/chess-cli view xyz.bin all
./target/release/chess-cli count xyz.bin

# Evaluate positions with NNUE and print scores
./target/release/chess-cli nnue nnue_trainer/nnue_model.safetensors xyz.bin 5
./target/release/chess-cli nnue nnue_trainer/nnue_model.safetensors xyz.bin 1-10
./target/release/chess-cli nnue nnue_trainer/nnue_model.safetensors xyz.bin all
```

#### Python trainer 

All paths configured in `nnue_trainer/configs/default.yaml`.
```bash 
cd nnue_trainer
pip install -r requirements.txt
# TODO .venv

# Generate training data (see CLI section)
# Should be able to train.py immediately (.bin file is in correct location)

# (Paths are in configs/)
python train.py
python check_model.py
python check_model.py --range "1-10,50-60"
python export_safetensors.py
```
