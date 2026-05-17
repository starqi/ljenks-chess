import argparse
from pathlib import Path
import sys

from safetensors.torch import save_file
from checkpoint import load_existing_model_only


def main():
    parser = argparse.ArgumentParser(description='Export NNUE checkpoint to safetensors')
    parser.add_argument('checkpoint', type=str, help='Path to checkpoint folder')
    args = parser.parse_args()

    model = load_existing_model_only(args.checkpoint)
    if model is None:
        print(f"Error: no valid checkpoint found at {args.checkpoint}")
        sys.exit(1)

    out_path = Path(args.checkpoint).parent / f"{args.checkpoint}.safetensors"
    state_dict = model.state_dict()

    print(f"Tensors: {list(state_dict.keys())}")
    for name, tensor in state_dict.items():
        print(f"  {name}: {tuple(tensor.shape)} {tensor.dtype}")

    save_file(state_dict, str(out_path))
    size_mb = out_path.stat().st_size / (1024 * 1024)
    print(f"Saved {out_path} ({size_mb:.2f} MB)")


if __name__ == "__main__":
    main()
