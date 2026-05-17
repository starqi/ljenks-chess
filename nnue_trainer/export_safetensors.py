from pathlib import Path
import sys

from safetensors.torch import save_file
from checkpoint import load_existing_model_only
from config import load_config


def main():
    config = load_config()
    checkpoint_path = Path(config['checkpoint_path'])
    model = load_existing_model_only(checkpoint_path)
    if model is None:
        print(f"Error: no valid checkpoint found at {checkpoint_path}")
        sys.exit(1)

    out_path = checkpoint_path.parent / f"{checkpoint_path.name}.safetensors"
    state_dict = model.state_dict()

    print(f"Tensors: {list(state_dict.keys())}")
    for name, tensor in state_dict.items():
        print(f"  {name}: {tuple(tensor.shape)} {tensor.dtype}")

    save_file(state_dict, str(out_path))
    size_mb = out_path.stat().st_size / (1024 * 1024)
    print(f"Saved {out_path} ({size_mb:.2f} MB)")


if __name__ == "__main__":
    main()
