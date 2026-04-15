import torch
import torch.nn as nn
from .dataset import NNUE_TOTAL_SIZE


class NNUE(nn.Module):

    def __init__(self, l1_size: int = 256, l2_size: int = 32):
        super().__init__()
        # More efficient way of doing l1_size by NNUE_TOTAL_SIZE sparse matrix instead of multiplying by 0 a gazillion times
        self.input: nn.EmbeddingBag = nn.EmbeddingBag(
            num_embeddings=NNUE_TOTAL_SIZE,
            embedding_dim=l1_size,
            mode='sum',
        )
        self.fc1: nn.Linear = nn.Linear(l1_size, l2_size)
        self.output: nn.Linear = nn.Linear(l2_size, 1)
    
        self._reset_weights()
    
    def _reset_weights(self):
        self.input.weight.data *= 512.0

    def forward(self, indices: torch.Tensor, offsets: torch.Tensor) -> torch.Tensor:
        x: torch.Tensor = self.input(indices, offsets)
        x = torch.nn.functional.leaky_relu(x, negative_slope=0.01)
        x = self.fc1(x)
        x = torch.nn.functional.leaky_relu(x, negative_slope=0.01)
        x = self.output(x)
        return x.squeeze(-1)
