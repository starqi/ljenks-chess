from math import sqrt
from pathlib import Path
import torch
from torch.optim import Adam
from model import NNUE
from dataset import create_dataloader

# Chess engines total tensor size not big enough for GPU
DEVICE = torch.device("cpu")


def create_model(existing_model_path: Path | None) -> NNUE :
    model = NNUE().to(DEVICE)
    if existing_model_path is not None and existing_model_path.exists():
        print(f"Loading existing model at {existing_model_path}")
        model.load_state_dict(torch.load(existing_model_path, map_location=DEVICE, weights_only=True))
    return model


def create_optimizer(model: NNUE, lr: float, existing_optimizer_path: Path | None) -> Adam:
    optimizer = Adam(model.parameters(), lr=lr, weight_decay=1e-5)
    if existing_optimizer_path is not None and existing_optimizer_path.exists():
        print(f"Loading existing optimizer at {existing_optimizer_path}")
        optimizer.load_state_dict(torch.load(existing_optimizer_path, map_location=DEVICE, weights_only=True))
    # TODO (Minor) Look up any Adam recommendations?
    # Basics: Per param optimizer, exponential weighted average, momentum + anti-oscillation
    return optimizer


def _forward_loss(model: NNUE, indices_stm, offsets_stm, indices_opp, offsets_opp, scores, sigmoid_scale: float):
    pred = model(indices_stm, offsets_stm, indices_opp, offsets_opp)
    pred_sig = torch.sigmoid(pred / sigmoid_scale)
    return torch.nn.functional.mse_loss(pred_sig, scores)


def train(
    positions_path: str,
    epochs: int,
    model: NNUE,
    optimizer: Adam,
    batch_size: int,
    sigmoid_scale: float,
    loader_num_workers: int):

    # TODO (Minor)
    #if seed is not None:
    #    torch.manual_seed(seed)

    dataloader = create_dataloader(
        positions_path,
        sigmoid_scale,
        batch_size,
        loader_num_workers,
        True
    )

    for epoch in range(epochs):
        total_loss = 0
        count = 0
        for indices_stm, offsets_stm, indices_opp, offsets_opp, scores in dataloader:
            indices_stm = indices_stm.to(DEVICE)
            offsets_stm = offsets_stm.to(DEVICE)
            indices_opp = indices_opp.to(DEVICE)
            offsets_opp = offsets_opp.to(DEVICE)
            scores = scores.to(DEVICE)

            optimizer.zero_grad()
            loss = _forward_loss(model, indices_stm, offsets_stm, indices_opp, offsets_opp, scores, sigmoid_scale)
            loss.backward()
            optimizer.step()

            total_loss += loss.item()
            count += 1
            if count % 1000 == 0:
                avg_loss = total_loss / count if count > 0 else 0
                print(f"Optimizer steps: {count}, avg loss: {avg_loss:.4f}, square root: {sqrt(avg_loss):.4f}")


def validate(model: NNUE, val_path: str, sigmoid_scale: float, batch_size: int, loader_num_workers: int) -> float:
    print("Validating")
    model.eval()
    dataloader = create_dataloader(val_path, sigmoid_scale, batch_size, loader_num_workers, False)
    total_loss = 0.0
    count = 0
    with torch.no_grad():
        for indices_stm, offsets_stm, indices_opp, offsets_opp, scores in dataloader:
            indices_stm = indices_stm.to(DEVICE)
            offsets_stm = offsets_stm.to(DEVICE)
            indices_opp = indices_opp.to(DEVICE)
            offsets_opp = offsets_opp.to(DEVICE)
            scores = scores.to(DEVICE)
            loss = _forward_loss(model, indices_stm, offsets_stm, indices_opp, offsets_opp, scores, sigmoid_scale)
            total_loss += loss.item()
            count += 1
    avg = total_loss / count if count > 0 else float("inf")
    model.train()
    print(f"Validation loss: {avg:.4f}, square root: {sqrt(avg):.4f}")
    return avg
