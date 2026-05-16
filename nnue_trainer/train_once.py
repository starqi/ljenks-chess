import os
import torch
from config import load_config
from model import NNUE
from trainer import get_device, train

if __name__ == '__main__':
    config = load_config()
    checkpoint_path = config['checkpoint_path']
    device = get_device()
    model = NNUE().to(device)
    if os.path.exists(checkpoint_path):
        model.load_state_dict(torch.load(checkpoint_path, map_location=device, weights_only=True))
        print(f"Loaded checkpoint from {checkpoint_path}")

    model = train(
        config['positions_path'],
        config['simple_epochs'],
        model,
        config['batch_size'],
        config['lr'],
    )

    torch.save(model.state_dict(), checkpoint_path)
    print(f"Model saved to {checkpoint_path}")
