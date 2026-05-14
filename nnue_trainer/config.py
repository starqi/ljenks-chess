import yaml
import os
from typing import TypedDict

_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

class Config(TypedDict):
    # Common
    batch_size: int
    lr: float
    save_path: str
    # train.py
    bin_path: str
    epochs: int
    # stream_train.py
    cycles: int
    epochs_per_cycle: int
    games_per_worker: int
    workers: int
    max_nodes: int
    random_half_moves: int
    max_half_moves: int
    val_every: int
    validation_path: str

def load_config() -> Config:
    with open(os.path.join(_SCRIPT_DIR, 'configs', 'default.yaml')) as f:
        config = yaml.safe_load(f)
    if not os.path.isabs(config['bin_path']):
        config['bin_path'] = os.path.join(_SCRIPT_DIR, config['bin_path'])
    if not os.path.isabs(config['save_path']):
        config['save_path'] = os.path.join(_SCRIPT_DIR, config['save_path'])
    if not os.path.isabs(config['validation_path']):
        config['validation_path'] = os.path.join(_SCRIPT_DIR, config['validation_path'])
    return config