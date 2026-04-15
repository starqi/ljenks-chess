import torch
from torch.optim import Adam
from .model import NNUE
from .dataset import create_dataloader


def get_device():
    if torch.cuda.is_available():
        return torch.device("cuda")
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")


def train(bin_path: str, epochs: int = 10, batch_size: int = 1024, lr: float = 1e-3):
    device = get_device()
    print(f"Using device: {device}")
    
    dataloader = create_dataloader(bin_path, batch_size=batch_size, num_workers=0)
    model = NNUE().to(device)
    optimizer = Adam(model.parameters(), lr=lr) # TODO IMMEDIATE Read again
    
    for epoch in range(epochs):
        total_loss = 0
        count = 0
        for indices, offsets, scores in dataloader:
            indices = indices.to(device)
            offsets = offsets.to(device)
            scores = scores.to(device)
            
            optimizer.zero_grad()
            pred = model(indices, offsets)
            loss = torch.nn.functional.mse_loss(pred, scores)
            loss.backward()
            optimizer.step()
            
            total_loss += loss.item()
            count += 1
            
            if count % 10 == 0:
                print(f"Epoch {epoch + 1}, Batch {count}, Loss: {loss.item():.4f}")
        
        avg_loss = total_loss / count if count > 0 else 0
        print(f"Epoch {epoch + 1} complete, Avg Loss: {avg_loss:.4f}")
