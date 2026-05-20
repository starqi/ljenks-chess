#!/usr/bin/env bash
# (GENERATED)
# Run on an Ubuntu VM to install everything and build the CLI.
# After this, follow the README commands as normal.
set -euo pipefail

echo "=== Updating system ==="
sudo apt update && sudo apt install -y \
  build-essential pkg-config libssl-dev python3 python3-venv git

echo "=== Installing Rust ==="
if command -v cargo &>/dev/null; then
  echo "Rust already installed: $(cargo --version)"
else
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

echo "=== Building chess-cli ==="
cd "$(dirname "$0")"
cargo build --release --bin chess-cli --no-default-features -F cli

echo "=== Setting up Python ==="
cd nnue_trainer
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip

if command -v nvidia-smi &>/dev/null; then
  echo "NVIDIA GPU detected, installing PyTorch (CUDA)..."
  pip install torch
else
  echo "No GPU detected, installing PyTorch (CPU)..."
  pip install torch --index-url https://download.pytorch.org/whl/cpu
fi

pip install -r requirements.txt

echo ""
echo "=== Setup complete ==="
echo "Now follow the README commands, e.g.:"
echo "  cd nnue_trainer && source .venv/bin/activate && python repeat_train.py"
