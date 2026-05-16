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
    epochs: int,
    model: NNUE,
    batch_size: int = 1024,
    lr: float = 1e-3,
) -> NNUE:

    # TODO (Minor)
    #if seed is not None:
    #    torch.manual_seed(seed)

    device = get_device()
    print(f"Training, using device: {device}")

    # TODO (Minor) Do we actually need workers to load?
    dataloader = create_dataloader(bin_path, batch_size=batch_size, num_workers=0) # num_workers = Parallel data load
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

    return model
