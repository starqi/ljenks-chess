import argparse
import sys

import torch
from checkpoint import load_existing_model_only
from dataset import ChessDataset

def parse_ranges(range_str: str) -> list[tuple[int, int]]:
    """Parse range string like '1-10', '1-10,50-60' into list of (0-based start, exclusive end) tuples."""
    ranges: list[tuple[int, int]] = []
    for part in range_str.split(','):
        part = part.strip()
        if '-' in part:
            s, e = part.split('-', 1)
            start = int(s) - 1
            end = int(e) # Convert from inclusive end to exclusive end
            ranges.append((start, end))
        else:
            idx = int(part) - 1 # crash if not a number
            ranges.append((idx, idx + 1))
    return ranges

parser = argparse.ArgumentParser(description='Check NNUE model predictions')
parser.add_argument('checkpoint', type=str, help='Path to checkpoint folder')
parser.add_argument('positions', type=str, help='Path to .bin positions file')
parser.add_argument('--range', type=str, default=None,
                    help='Position range, e.g. "1-10", "1-10,50-60", "5" (1-based)')
parser.add_argument('--sigmoid', type=int, default=None)
args = parser.parse_args()

model = load_existing_model_only(args.checkpoint)
if model is None:
    print(f"Error: no checkpoint found at {args.checkpoint}")
    sys.exit(1)
model.eval()

dataset = ChessDataset(args.positions, None)
ranges = parse_ranges(args.range) if args.range else [(0, len(dataset))]

for start, end in ranges:
    for idx in range(start, end):
        if idx >= len(dataset):
            print(f"Position {idx + 1} is out of range (file has {len(dataset)} positions)")
            continue
        idx_stm, idx_opp, score = dataset[idx]
        pred: torch.Tensor = model(idx_stm, torch.tensor([0], dtype=torch.long), idx_opp, torch.tensor([0], dtype=torch.long))
        pred_val = pred.item()
        if args.sigmoid is not None:
            pred_sig = torch.sigmoid(torch.tensor(pred_val / args.sigmoid)).item()
            target_sig = torch.sigmoid(torch.tensor(score / args.sigmoid)).item()
            print(f"Position {idx + 1} - Pred: {pred_sig:.4f} - Target: {target_sig:.4f} - Diff: {abs(pred_sig - target_sig):.4f}")
        else:
            print(f"Position {idx + 1} - Pred: {pred_val:.1f} - Target: {score:.1f} - Diff: {abs(pred_val - score):.1f}")
