import torch
import torch.nn as nn
from .dataset import NNUE_HALF_SIZE


class NNUE(nn.Module):

    # TODO (Read) Some intuition around possible vector sizes
    def __init__(self, l1_size: int = 256, l2_size: int = 32):
        super().__init__()
        # EmbeddingBag intuition:
        # More efficient way of doing l1_size by NNUE_TOTAL_SIZE sparse matrix instead of multiplying by 0 a gazillion times

        # Shared weight table for both accumulators
        self.input: nn.EmbeddingBag = nn.EmbeddingBag(
            num_embeddings=NNUE_HALF_SIZE,
            embedding_dim=l1_size,
            mode='sum',
        )
        # 2 accumulators concat together and then output l2_size
        self.fc1: nn.Linear = nn.Linear(2 * l1_size, l2_size)
        self.output: nn.Linear = nn.Linear(l2_size, 1)

        self._reset_weights()

    def _reset_weights(self):
        self.input.weight.data *= 512.0

    def forward(self, 
                indices_stm: torch.Tensor, offsets_stm: torch.Tensor,
                indices_opp: torch.Tensor, offsets_opp: torch.Tensor) -> torch.Tensor:

        # stm = Side to move, opp = opponent
        x_stm: torch.Tensor = self.input(indices_stm, offsets_stm)
        x_opp: torch.Tensor = self.input(indices_opp, offsets_opp)
        print('TODO', x_stm.shape)
        x = torch.cat([x_stm, x_opp], dim=1)
        # Ran into dead ReLU immediately and fixed with leaky ReLU
        x = torch.nn.functional.leaky_relu(x, negative_slope=0.01)
        x = self.fc1(x)
        x = torch.nn.functional.leaky_relu(x, negative_slope=0.01)
        x = self.output(x)
        return x.squeeze(-1)
