import torch
from torch.utils.data import Dataset, DataLoader
import numpy as np
from numpy.typing import NDArray
from pathlib import Path
from typing import BinaryIO

COMPRESSED_SIZE = 34
RECORD_SIZE = 38

# Constants from Rust engine
NNUE_PIECE_FEATURES = 64 * 64 * 12
NNUE_CASTLE_FEATURES = 2
NNUE_EP_FEATURES = 8
NNUE_HALF_SIZE = NNUE_PIECE_FEATURES + NNUE_CASTLE_FEATURES + NNUE_EP_FEATURES
NNUE_TOTAL_SIZE = 2 * NNUE_HALF_SIZE


def piece_to_nnue_index(piece: int, player: int, perspective: int) -> int:
    # TODO IMMEDIATE I assume this pointless looking translation layer is because the nnue.rs and compressed.rs are 
    # enumerated differently, make them all match the Piece enum @ entities.rs
    mapping = {0: 0, 2: 1, 3: 2, 1: 3, 4: 4, 5: 5}
    base = mapping[piece]
    return base + (6 if player != perspective else 0)


# TODO IMMEDIATE Refactor all the crazy types to make more sense for a human
def decode_compressed(data: bytes) -> tuple[list[tuple[int, int]], list[tuple[int, int]], int, int, int, int, int, int]:
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
    w_oo = (flags >> 6) & 1
    w_ooo = (flags >> 5) & 1
    b_oo = (flags >> 4) & 1
    b_ooo = (flags >> 3) & 1
    
    ep_code = data[33]
    ep_file = ep_code - 1 if ep_code > 0 else -1
    
    return white_pieces, black_pieces, side_to_move, w_oo, w_ooo, b_oo, b_ooo, ep_file


def compute_nnue_indices(data: bytes) -> list[int]:
    white_pieces, black_pieces, side_to_move, w_oo, w_ooo, b_oo, b_ooo, ep_file = decode_compressed(data)
    
    indices: list[int] = []
    
    # TODO IMMEDIATE Fix magic number 5 after fixing piece_to_nnue_index
    white_king_sq = next((sq for sq, p in white_pieces if p == 5), None)
    black_king_sq = next((sq for sq, p in black_pieces if p == 5), None)
    
    if white_king_sq is None or black_king_sq is None:
        raise ValueError("Missing king")
    
    # Side to move perspective (top half of vector)
    perspective = side_to_move
    king_sq = white_king_sq if perspective == 0 else black_king_sq ^ 56
    flip_mask = 0 if perspective == 0 else 56
    
    for sq, piece in white_pieces:
        piece_idx = piece_to_nnue_index(piece, 0, perspective)
        sq_idx = sq ^ flip_mask
        bucket = king_sq * 64 * 12 + sq_idx * 12 + piece_idx
        indices.append(bucket)
    
    for sq, piece in black_pieces:
        piece_idx = piece_to_nnue_index(piece, 1, perspective)
        sq_idx = sq ^ flip_mask
        bucket = king_sq * 64 * 12 + sq_idx * 12 + piece_idx
        indices.append(bucket)
    
    castle_offset = NNUE_PIECE_FEATURES
    if perspective == 0:
        if w_oo: indices.append(castle_offset + 0)
        if w_ooo: indices.append(castle_offset + 1)
    else:
        if b_oo: indices.append(castle_offset + 0)
        if b_ooo: indices.append(castle_offset + 1)
    
    ep_offset = NNUE_PIECE_FEATURES + NNUE_CASTLE_FEATURES
    if ep_file >= 0:
        indices.append(ep_offset + ep_file)

    # TODO IMMEDIATE Find a way to share code between above and below
    
    # Opponent perspective (bottom half of vector)
    opp_perspective = 1 - side_to_move
    opp_king_sq = black_king_sq if opp_perspective == 0 else white_king_sq ^ 56
    opp_flip_mask = 0 if opp_perspective == 0 else 56
    
    for sq, piece in white_pieces:
        piece_idx = piece_to_nnue_index(piece, 0, opp_perspective)
        sq_idx = sq ^ opp_flip_mask
        bucket = opp_king_sq * 64 * 12 + sq_idx * 12 + piece_idx
        indices.append(NNUE_HALF_SIZE + bucket)
    
    for sq, piece in black_pieces:
        piece_idx = piece_to_nnue_index(piece, 1, opp_perspective)
        sq_idx = sq ^ opp_flip_mask
        bucket = opp_king_sq * 64 * 12 + sq_idx * 12 + piece_idx
        indices.append(NNUE_HALF_SIZE + bucket)
    
    if opp_perspective == 0:
        if w_oo: indices.append(NNUE_HALF_SIZE + castle_offset + 0)
        if w_ooo: indices.append(NNUE_HALF_SIZE + castle_offset + 1)
    else:
        if b_oo: indices.append(NNUE_HALF_SIZE + castle_offset + 0)
        if b_ooo: indices.append(NNUE_HALF_SIZE + castle_offset + 1)
    
    return indices


class ChessDataset(Dataset[tuple[torch.Tensor, float]]):
    def __init__(self, bin_path: str, max_positions: int | None = None):
        self.bin_path: Path = Path(bin_path)
        if not self.bin_path.exists():
            raise FileNotFoundError(f"Binary file not found: {bin_path}")
        self.file_size: int = self.bin_path.stat().st_size
        self.num_positions: int = self.file_size // RECORD_SIZE
        if max_positions:
            self.num_positions = min(self.num_positions, max_positions)
        
        self._file: BinaryIO | None = None
        self._mmap: NDArray[np.uint8] | None = None
    
    def _init_mmap(self):
        if self._mmap is None:
            self._file = open(self.bin_path, 'rb')
            # TODO IMMEDIATE Review/comment on memmap
            self._mmap = np.memmap(self._file, dtype=np.uint8, mode='r') # Numpy will help create a ndarray looking buffer using memory maps
    
    def __len__(self):
        return self.num_positions
    
    def __getitem__(self, idx: int) -> tuple[torch.Tensor, float]:
        self._init_mmap()
        assert self._mmap is not None
        
        offset = idx * RECORD_SIZE
        board_data = bytes(self._mmap[offset:offset + COMPRESSED_SIZE])
        score_bytes = self._mmap[offset + COMPRESSED_SIZE:offset + RECORD_SIZE]
        score = np.frombuffer(score_bytes, dtype='<i4')[0].item()
        score = max(-32000, min(32000, score))
        
        indices = compute_nnue_indices(board_data)
        indices_tensor = torch.tensor(indices, dtype=torch.long)
        
        return indices_tensor, float(score)
    
    def close(self):
        if self._mmap is not None:
            del self._mmap
            self._mmap = None
        if self._file is not None:
            self._file.close()
            self._file = None


# Makes batches
def collate_fn(batch: list[tuple[torch.Tensor, float]]) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    idx_tensors: list[torch.Tensor] = []
    sample_offsets: list[int] = []
    sample_scores: list[float] = []
    current_offset = 0
    for idx_tensor, score in batch:
        idx_tensors.append(idx_tensor)
        sample_offsets.append(current_offset)
        current_offset += len(idx_tensor)
        sample_scores.append(score)
    
    idx_batch_tensor = torch.cat(idx_tensors) # Typical 1D array concat
    offsets_tensor = torch.tensor(sample_offsets, dtype=torch.long) # Starting positions of each item in the concated 1D array 
    scores_tensor = torch.tensor(sample_scores, dtype=torch.float32)
    
    return idx_batch_tensor, offsets_tensor, scores_tensor


# TODO ...ljenks-chess/nnue_trainer/.venv/lib/python3.10/site-packages/torch/utils/data/dataloader.py:775:
# UserWarning: 'pin_memory' argument is set
# as true but not supported on MPS now, device pinned memory won't be used.
def create_dataloader(bin_path: str, batch_size: int = 1024, max_positions: int | None = None, num_workers: int = 0, shuffle: bool = True) -> DataLoader[tuple[torch.Tensor, float]]:
    dataset = ChessDataset(bin_path, max_positions)
    # "Pinned memory in PyTorch acts like disabling virtual memory
    #  for that specific tensor by creating "page-locked" CPU memory that the OS cannot swap to disk.
    #  This ensures faster GPU transfers (Direct Memory Access) but doesn't disable virtual memory system-wide,
    #  instead locking specific data in physical RAM."
    return DataLoader(dataset, batch_size=batch_size, shuffle=shuffle, collate_fn=collate_fn, num_workers=num_workers, pin_memory=True)

