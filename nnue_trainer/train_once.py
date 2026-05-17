from checkpoint import load_checkpoint, rotate_and_save
from config import load_config
from trainer import train

if __name__ == '__main__':
    config = load_config()
    checkpoint = load_checkpoint(config)

    train(
        config['positions_path'],
        config['simple_epochs'],
        checkpoint.model,
        checkpoint.optimizer,
        config['batch_size'],
        config['sigmoid_scale'],
    )

    rotate_and_save(config, checkpoint)
