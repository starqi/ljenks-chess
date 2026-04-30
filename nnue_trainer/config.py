import yaml
import os

_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

def load_config() -> dict:
    with open(os.path.join(_SCRIPT_DIR, 'configs', 'default.yaml')) as f:
        config = yaml.safe_load(f)
    if not os.path.isabs(config['bin_path']):
        config['bin_path'] = os.path.join(_SCRIPT_DIR, config['bin_path'])
    if not os.path.isabs(config['save_path']):
        config['save_path'] = os.path.join(_SCRIPT_DIR, config['save_path'])
    return config
