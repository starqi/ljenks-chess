import os
import sys

# Call as script from anywhere
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.insert(0, script_dir)

from config import load_config
from trainer import train

if __name__ == '__main__':
    config = load_config()
    train(
        bin_path=config['bin_path'],
        epochs=config['epochs'],
        batch_size=config['batch_size'],
        lr=config['lr'],
        save_path=config['save_path'],
    )