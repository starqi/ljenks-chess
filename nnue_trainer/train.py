# TODO IMMEDIATE This training doesn't actually work btw, error doesn't go down

import yaml
import os
import sys

# Call as script from anywhere
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.insert(0, script_dir)

from src.trainer import train


if __name__ == '__main__':
    config_path = os.path.join(script_dir, 'src', 'configs', 'default.yaml')
    with open(config_path) as f:
        config = yaml.safe_load(f)
    if not os.path.isabs(config['bin_path']):
        config['bin_path'] = os.path.join(script_dir, config['bin_path'])
    
    train(**config)
