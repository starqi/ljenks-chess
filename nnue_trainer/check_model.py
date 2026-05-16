import argparse

import torch
from config import load_config
from model import NNUE
from dataset import ChessDataset

def parse_ranges(range_str):
    """Parse range string like '1-10', '1-10,50-60' into list of (0-based start, exclusive end) tuples."""
    ranges = []
    for part in range_str.split(','):
        part = part.strip()
        if '-' in part:
            s, e = part.split('-', 1)
            start = int(s) - 1
            end = int(e) # Convert from inclusive end to exclusive end
            ranges.append((start, end))
        else:
            idx = int(part) - 1 # Crash if not a number
            ranges.append((idx, idx + 1))
    return ranges

config = load_config()

parser = argparse.ArgumentParser(description='Check NNUE model predictions')
parser.add_argument('--range', type=str, default=None,
                    help='Position range, e.g. "1-10", "1-10,50-60", "5" (1-based)')
args = parser.parse_args()

model = NNUE()
model.load_state_dict(torch.load(config['checkpoint_path'], weights_only=True))
model.eval()

dataset = ChessDataset(config['positions_path'])
ranges = parse_ranges(args.range) if args.range else [(0, len(dataset))]

for start, end in ranges:
    for idx in range(start, end):
        if idx >= len(dataset):
            print(f"Position {idx + 1} is out of range (file has {len(dataset)} positions)")
            continue
        idx_stm, idx_opp, score = dataset[idx]
        pred = model(idx_stm, torch.tensor([0], dtype=torch.long), idx_opp, torch.tensor([0], dtype=torch.long))
        print(f"Position {idx + 1} - Pred: {pred.item():.1f} - Target: {score:.1f}")