import yaml
import os
from typing import Any, TypedDict

_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

def _resolve_worker_count(value: Any) -> int:
    if isinstance(value, int):
        return value
    elif value == "full":
        return os.cpu_count() or 1
    elif value == "half":
        return (os.cpu_count() or 2) // 2
    else:
        return 1

# All paths are allowed to not exist at time of config parse
class Config(TypedDict):

    # Common
    batch_size: int
    lr: float
    checkpoint_path: str
    loader_num_workers: int

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
    sigmoid_scale: float


def load_config() -> Config:
    with open(os.path.join(_SCRIPT_DIR, 'configs', 'default.yaml')) as f:
        config = yaml.safe_load(f)
    if not os.path.isabs(config['positions_path']):
        config['positions_path'] = os.path.join(_SCRIPT_DIR, config['positions_path'])
    if not os.path.isabs(config['checkpoint_path']):
        config['checkpoint_path'] = os.path.join(_SCRIPT_DIR, config['checkpoint_path'])
    if not os.path.isabs(config['validation_path']):
        config['validation_path'] = os.path.join(_SCRIPT_DIR, config['validation_path'])
    if config['checkpoint_backup_count'] < 0:
        raise ValueError(f"checkpoint_backup_count cannot be negative: {config['checkpoint_backup_count']}")
    config['loader_num_workers'] = _resolve_worker_count(config['loader_num_workers'])
    config['workers'] = _resolve_worker_count(config['workers'])
    print(f"Loaded config: {config!r}")
    return config
