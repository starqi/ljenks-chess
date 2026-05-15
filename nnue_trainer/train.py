import os
import torch
from config import load_config
from model import NNUE
from trainer import get_device, train

if __name__ == '__main__':
    config = load_config()
    save_path = config['save_path']
    device = get_device()
    model = NNUE().to(device)
    if os.path.exists(save_path):
        model.load_state_dict(torch.load(save_path, map_location=device, weights_only=True))
        print(f"Loaded checkpoint from {save_path}")

    model = train(
        config['bin_path'],
        config['epochs'],
        model,
        config['batch_size'],
        config['lr'],
    )

    torch.save(model.state_dict(), save_path)
    print(f"Model saved to {save_path}")
