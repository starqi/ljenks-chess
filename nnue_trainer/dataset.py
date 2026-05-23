from typing_extensions import override
import torch
from torch.utils.data import Dataset, DataLoader
import numpy as np
from numpy.typing import NDArray
from pathlib import Path
from typing import BinaryIO, NamedTuple
from math import exp

COMPRESSED_SIZE = 34
RECORD_SIZE = 38

NNUE_PIECE_FEATURES = 64 * 64 * 12
NNUE_CASTLE_FEATURES = 2
NNUE_EP_FEATURES = 8
NNUE_HALF_SIZE = NNUE_PIECE_FEATURES + NNUE_CASTLE_FEATURES + NNUE_EP_FEATURES


class DecodedBoard(NamedTuple):
    # List of (index, piece), where index is white perspective, no ^56 in encoding stage, Piece is `Piece` enum number
    white_pieces: list[tuple[int, int]]
    black_pieces: list[tuple[int, int]]
    # 0, 1 == white, black
    side_to_move: int
    # 0, 1 == false, true
    w_oo_moved: int
    w_ooo_moved: int
    b_oo_moved: int
    b_ooo_moved: int
    ep_file: int  # -1 if none


def decode_compressed(data: bytes) -> DecodedBoard:
    """Decodes Rust side compressed.rs format."""
    white_pieces: list[tuple[int, int]] = []
    black_pieces: list[tuple[int, int]] = []
    for i in range(64):
        nibble = (data[i // 2] >> ((i & 1) * 4)) & 0x0F
        if nibble > 0:
            code = nibble - 1
            piece = code % 6
            player = code // 6
            if player == 0:
                white_pieces.append((i, piece))
            else:
                black_pieces.append((i, piece))

    flags = data[32]
    side_to_move = (flags >> 7) & 1
    w_oo_moved = (flags >> 6) & 1
    w_ooo_moved = (flags >> 5) & 1
    b_oo_moved = (flags >> 4) & 1
    b_ooo_moved = (flags >> 3) & 1

    ep_code = data[33]
    ep_file = ep_code - 1 if ep_code > 0 else -1

    return DecodedBoard(
        white_pieces=white_pieces,
        black_pieces=black_pieces,
        side_to_move=side_to_move,
        w_oo_moved=w_oo_moved,
        w_ooo_moved=w_ooo_moved,
        b_oo_moved=b_oo_moved,
        b_ooo_moved=b_ooo_moved,
        ep_file=ep_file,
    )


def compute_half_indices(perspective: int, board: DecodedBoard) -> list[int]:
    KING_PIECE = 5
    white_king_sq = next((sq for sq, p in board.white_pieces if p == KING_PIECE), None)
    black_king_sq = next((sq for sq, p in board.black_pieces if p == KING_PIECE), None)
    if white_king_sq is None or black_king_sq is None:
        raise ValueError("Missing king")

    king_sq = white_king_sq if perspective == 0 else black_king_sq ^ 56
    flip_mask = 0 if perspective == 0 else 56

    indices: list[int] = []
    for sq, piece in board.white_pieces:
        piece_idx = piece + (6 if 0 != perspective else 0)
        sq_idx = sq ^ flip_mask
        indices.append(king_sq * 64 * 12 + sq_idx * 12 + piece_idx)

    for sq, piece in board.black_pieces:
        piece_idx = piece + (6 if 1 != perspective else 0)
        sq_idx = sq ^ flip_mask
        indices.append(king_sq * 64 * 12 + sq_idx * 12 + piece_idx)

    castle_offset = NNUE_PIECE_FEATURES
    if perspective == 0:
        if board.w_oo_moved: indices.append(castle_offset + 0)
        if board.w_ooo_moved: indices.append(castle_offset + 1)
    else:
        if board.b_oo_moved: indices.append(castle_offset + 0)
        if board.b_ooo_moved: indices.append(castle_offset + 1)

    if perspective == board.side_to_move and board.ep_file >= 0:
        ep_offset = NNUE_PIECE_FEATURES + NNUE_CASTLE_FEATURES
        indices.append(ep_offset + board.ep_file)

    return indices


def compute_nnue_indices(data: bytes) -> tuple[list[int], list[int]]:
    board = decode_compressed(data)
    stm = board.side_to_move
    assert stm == 0 or stm == 1
    return compute_half_indices(stm, board), compute_half_indices(1 - stm, board)


def sigmoid_cp(score: float, scale: float) -> float:
    x = score / scale
    if x >= 0:
        return 1.0 / (1.0 + exp(-x))
    return exp(x) / (1.0 + exp(x))


class ChessDataset(Dataset[tuple[torch.Tensor, torch.Tensor, float]]):

    # Allow disabling the sigmoid to view dataset in centipawns
    def __init__(self, positions_path: str, sigmoid_scale: float | None, max_positions: int | None = None):
        self.positions_path: Path = Path(positions_path)
        self.sigmoid_scale: float | None = sigmoid_scale
        if not self.positions_path.exists():
            raise FileNotFoundError(f"Binary file not found: {positions_path}")
        self.file_size: int = self.positions_path.stat().st_size
        self.num_positions: int = self.file_size // RECORD_SIZE
        if max_positions:
            self.num_positions = min(self.num_positions, max_positions)

        self._file: BinaryIO | None = None
        self._mmap: NDArray[np.uint8] | None = None

    def _init_mmap(self):
        if self._mmap is None:
            self._file = open(self.positions_path, 'rb')
            self._mmap = np.memmap(self._file, dtype=np.uint8, mode='r')

    def __len__(self):
        return self.num_positions

    @override
    def __getitem__(self, idx: int) -> tuple[torch.Tensor, torch.Tensor, float]:
        self._init_mmap()
        assert self._mmap is not None

        offset = idx * RECORD_SIZE
        board_data = bytes(self._mmap[offset:offset + COMPRESSED_SIZE])
        score_bytes = self._mmap[offset + COMPRESSED_SIZE:offset + RECORD_SIZE]
        score = np.frombuffer(score_bytes, dtype='<i4')[0].item() # < means little endian
        score = max(-32000, min(32000, score))
        if self.sigmoid_scale is not None:
            score = sigmoid_cp(score, self.sigmoid_scale)

        indices_stm, indices_opp = compute_nnue_indices(board_data)

        return torch.tensor(indices_stm, dtype=torch.long), torch.tensor(indices_opp, dtype=torch.long), float(score)

    def close(self):
        if self._mmap is not None:
            del self._mmap
            self._mmap = None
        if self._file is not None:
            self._file.close()
            self._file = None


# Torch basics: Forward pass will see the collated version of data
def collate_fn(batch: list[tuple[torch.Tensor, torch.Tensor, float]]) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    idx_stm_tensors: list[torch.Tensor] = []
    idx_opp_tensors: list[torch.Tensor] = []
    stm_offsets: list[int] = []
    opp_offsets: list[int] = []
    sample_scores: list[float] = []
    stm_offset = 0
    opp_offset = 0
    for idx_stm, idx_opp, score in batch:
        idx_stm_tensors.append(idx_stm)
        idx_opp_tensors.append(idx_opp)
        stm_offsets.append(stm_offset)
        opp_offsets.append(opp_offset)
        stm_offset += len(idx_stm)
        opp_offset += len(idx_opp)
        sample_scores.append(score)

    #print('collate_fn SHAPE DEBUG', idx_stm_tensors[0].shape)
    return ( # TODO IMMEDIATE Named tuple for this?
        torch.cat(idx_stm_tensors),
        torch.tensor(stm_offsets, dtype=torch.long),
        torch.cat(idx_opp_tensors),
        torch.tensor(opp_offsets, dtype=torch.long),
        torch.tensor(sample_scores, dtype=torch.float32),
    )


# TODO ...ljenks-chess/nnue_trainer/.venv/lib/python3.10/site-packages/torch/utils/data/dataloader.py:775:
# UserWarning: 'pin_memory' argument is set
# as true but not supported on MPS now, device pinned memory won't be used.
def create_dataloader(
    positions_path: str,
    sigmoid_scale: float,
    batch_size: int,
    num_workers: int,
    shuffle: bool) -> DataLoader[tuple[torch.Tensor, torch.Tensor, float]]:

    dataset = ChessDataset(positions_path, sigmoid_scale, None)
    # "Pinned memory in PyTorch acts like disabling virtual memory
    #  for that specific tensor by creating "page-locked" CPU memory that the OS cannot swap to disk.
    #  This ensures faster GPU transfers (Direct Memory Access) but doesn't disable virtual memory system-wide,
    #  instead locking specific data in physical RAM."
    return DataLoader(dataset, batch_size=batch_size, shuffle=shuffle, collate_fn=collate_fn, num_workers=num_workers, pin_memory=True)
