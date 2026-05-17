from dataclasses import dataclass
import json
from pathlib import Path
import shutil
import signal

import torch
from torch.optim import Adam
from config import Config
from model import NNUE
from trainer import create_model, create_optimizer

MODEL_FILE_NAME = "model.pt"
OPTIMIZER_FILE_NAME = "optimizer.pt"
STATE_FILE_NAME = "state.json"
STATE_CYCLE_KEY = "cycle"


@dataclass
class Checkpoint:
    model: NNUE
    optimizer: Adam
    cycle: int


def load_checkpoint(config: Config) -> Checkpoint:
    """Loads checkpoint from disk. Returns a fresh Checkpoint if path doesn't exist yet."""
    checkpoint_path = Path(config['checkpoint_path'])
    if checkpoint_path.exists() and not checkpoint_path.is_dir():
        raise NotADirectoryError(f"Checkpoint path must be a directory: {checkpoint_path}")
    if not checkpoint_path.exists():
        model = create_model(None)
        optimizer = create_optimizer(model, config['lr'], None)
        return Checkpoint(model, optimizer, cycle=0)
    model = create_model(checkpoint_path / MODEL_FILE_NAME)
    optimizer = create_optimizer(model, config['lr'], checkpoint_path / OPTIMIZER_FILE_NAME)
    cycle = 0
    state_json_path = checkpoint_path / STATE_FILE_NAME
    if state_json_path.exists():
        state_json = json.loads(state_json_path.read_text())
        if isinstance(state_json, dict):
            _cycle = state_json.get(STATE_CYCLE_KEY) # Right now this is the only key we have
            if isinstance(_cycle, int):
                cycle = _cycle
    return Checkpoint(model, optimizer, cycle)


def load_existing_model_only(checkpoint_path: Path | str) -> NNUE | None:
    """Loads only the model from a checkpoint directory. Returns None if no model."""
    checkpoint_path = Path(checkpoint_path)
    if not checkpoint_path.exists():
        return None
    if not checkpoint_path.is_dir():
        return None

    existing_model_path = checkpoint_path / MODEL_FILE_NAME 
    return create_model(existing_model_path) if existing_model_path.exists() else None


def save_checkpoint(checkpoint_dir: Path, mutated_checkpoint: Checkpoint):
    """Given an unused `checkpoint_dir` destination, writes `mutated_checkpoint` where cycles/model/optimizer might have been trained/changed"""
    if checkpoint_dir.exists():
        raise RuntimeError(f"Checkpoint path already exists, expected any existing checkpoints to have been renamed already: {checkpoint_dir}")
    checkpoint_dir.mkdir()
    torch.save(mutated_checkpoint.model.state_dict(), checkpoint_dir / MODEL_FILE_NAME)
    torch.save(mutated_checkpoint.optimizer.state_dict(), checkpoint_dir / OPTIMIZER_FILE_NAME)
    state_path = checkpoint_dir / STATE_FILE_NAME
    state_path.write_text(json.dumps({STATE_CYCLE_KEY: mutated_checkpoint.cycle}))


def _delete_file_or_folder(path: Path):
    if path.is_dir():
        shutil.rmtree(path)
    elif path.exists():
        path.unlink()


def rotate_and_save(config: Config, mutated_checkpoint: Checkpoint):
    # TODO IMMEDIATE What if backup count changes?
    checkpoint_dir = Path(config['checkpoint_path'])
    backup_count: int = config['checkpoint_backup_count']
    if backup_count < 0:
        raise ValueError(f"UNEXPECTED checkpoint_backup_count should have been checked to be non-negative: {backup_count}")
    old_handler = signal.signal(signal.SIGINT, signal.SIG_IGN)
    try:
        if backup_count == 0:
            if checkpoint_dir.exists():
                print(f"Deleting {checkpoint_dir}")
                _delete_file_or_folder(checkpoint_dir)
        else:
            oldest = Path(f"{checkpoint_dir}.bak.{backup_count}")
            if oldest.exists():
                print(f"Deleting {oldest}")
                _delete_file_or_folder(oldest)
            for i in range(backup_count - 1, 0, -1):
                src = Path(f"{checkpoint_dir}.bak.{i}")
                if src.exists():
                    dst = Path(f"{checkpoint_dir}.bak.{i + 1}")
                    print(f"{src} -> {dst}")
                    src.rename(dst)
            if checkpoint_dir.exists():
                dst = Path(f"{checkpoint_dir}.bak.1")
                print(f"{checkpoint_dir} -> {dst}")
                checkpoint_dir.rename(dst)
        print(f"Saving to {checkpoint_dir}")
        save_checkpoint(checkpoint_dir, mutated_checkpoint)
    finally:
        signal.signal(signal.SIGINT, old_handler)
