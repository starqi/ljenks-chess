import yaml
import os
from typing import TypedDict

_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

class Config(TypedDict):

    # Common
    batch_size: int
    lr: float
    checkpoint_path: str

    # train_once.py
    positions_path: str
    simple_epochs: int

    # repeat_train.py
    cycles: int
    epochs_per_cycle: int
    games_per_worker: int
    games_per_worker_validation_set: int
    workers: int
    max_nodes: int
    random_half_moves: int
    max_half_moves: int
    val_every: int
    validation_path: str
    checkpoint_backup_count: int


def load_config() -> Config:
    with open(os.path.join(_SCRIPT_DIR, 'configs', 'default.yaml')) as f:
        config = yaml.safe_load(f)
    if not os.path.isabs(config['positions_path']):
        config['positions_path'] = os.path.join(_SCRIPT_DIR, config['positions_path'])
    if not os.path.isabs(config['checkpoint_path']):
        config['checkpoint_path'] = os.path.join(_SCRIPT_DIR, config['checkpoint_path'])
    if not os.path.isabs(config['validation_path']):
        config['validation_path'] = os.path.join(_SCRIPT_DIR, config['validation_path'])
    return config
