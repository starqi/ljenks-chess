
import os
import sys

# Call as script from anywhere
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.insert(0, script_dir)

import torch
import pathlib
from src.model import NNUE
from src.dataset import create_dataloader

script_dir_path = pathlib.Path(script_dir)

model = NNUE()
model.load_state_dict(torch.load(script_dir_path / 'nnue_model.pt', weights_only=True))
model.eval()
loader = create_dataloader(str(script_dir_path / 'abc.bin'), batch_size=5, shuffle=False)

count = 0
for indices_stm, offsets_stm, indices_opp, offsets_opp, scores in loader:
    pred = model(indices_stm, offsets_stm, indices_opp, offsets_opp)
    print(f'Pred: {[round(p,1) for p in pred.tolist()]}')
    print(f'Target: {[round(s,1) for s in scores.tolist()]}')
    print()
    count += 1
    if count >= 10:
        break
