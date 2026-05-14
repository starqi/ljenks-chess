import os
import torch
from torch.optim import Adam
from model import NNUE
from dataset import create_dataloader

# TODO (Minor) Integer quantized READ
# TODO Fix basedpyright

# Chess engines total tensor size not big enough for GPU
def get_device():
    return torch.device("cpu")


def train(
    bin_path: str,
    epochs: int = 50,
    batch_size: int = 1024,
    lr: float = 1e-3,
    save_path: str | None = None,
    seed: int | None = None,
    model: NNUE | None = None,
    # TODO IMMEDIATE ALL DEVICE OBJS JUST COME FROM THE SAME FUCKING GET_DEVICE ABOVE. For chess engine, should we just hard code "cpu" in all cases?
    device: torch.device | None = None) -> NNUE:

    if seed is not None:
        torch.manual_seed(seed)

    device = device or get_device()
    print(f"Using device: {device}")

    # TODO IMMEDIATE num_workers=0?
    dataloader = create_dataloader(bin_path, batch_size=batch_size, num_workers=0)
    if model is None:
        model = NNUE().to(device)
        if save_path and os.path.exists(save_path):
            # TODO (Read) And extract, @stream_train.py
            model.load_state_dict(torch.load(save_path, map_location=device, weights_only=True))
            print(f"Loaded checkpoint from {save_path}")
    optimizer = Adam(model.parameters(), lr=lr, weight_decay=1e-5) # TODO (Read)

    for epoch in range(epochs):
        total_loss = 0
        count = 0
        for indices_stm, offsets_stm, indices_opp, offsets_opp, scores in dataloader:
            indices_stm = indices_stm.to(device)
            offsets_stm = offsets_stm.to(device)
            indices_opp = indices_opp.to(device)
            offsets_opp = offsets_opp.to(device)
            scores = scores.to(device)

            optimizer.zero_grad()
            pred = model(indices_stm, offsets_stm, indices_opp, offsets_opp)
            loss = torch.nn.functional.mse_loss(pred, scores)
            loss.backward()
            optimizer.step()

            total_loss += loss.item()
            count += 1

            if count % 10 == 0:
                print(f"Epoch {epoch + 1}, Batch {count}, Loss: {loss.item():.4f}")

        avg_loss = total_loss / count if count > 0 else 0
        print(f"Epoch {epoch + 1} complete, Avg Loss: {avg_loss:.4f}")

    if save_path:
        torch.save(model.state_dict(), save_path)
        print(f"Model saved to {save_path}")

    return model
