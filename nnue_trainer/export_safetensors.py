from pathlib import Path
import torch
from safetensors.torch import save_file
from config import load_config
import sys


def main():
    config = load_config()
    pt_path = Path(config['checkpoint_path'])
    if not pt_path.exists():
        print(f"Error: {pt_path} not found")
        sys.exit(1)

    out_path = pt_path.parent / f"{pt_path.stem}.safetensors"
    print(f"Loading {pt_path} ...")
    state_dict = torch.load(pt_path, map_location="cpu", weights_only=True)

    print(f"Tensors: {list(state_dict.keys())}")
    for name, tensor in state_dict.items():
        print(f"  {name}: {tuple(tensor.shape)} {tensor.dtype}")

    save_file(state_dict, str(out_path))
    size_mb = out_path.stat().st_size / (1024 * 1024)
    print(f"Saved {out_path} ({size_mb:.2f} MB)")


if __name__ == "__main__":
    main()
