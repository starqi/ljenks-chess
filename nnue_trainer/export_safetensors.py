"""
Export trained NNUE model from .pt to .safetensors format.
Outputs to the same directory as this script.

Usage: python export_safetensors.py [path_to_pt_file]

Default path: nnue_model.pt (same directory as script)
"""

import sys
import os

# Call as script from anywhere
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.insert(0, script_dir)

from pathlib import Path

import torch
from safetensors.torch import save_file

DEFAULT_PT = Path(__file__).parent / "nnue_model.pt"


def main():
    pt_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PT
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
