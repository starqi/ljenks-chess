use super::*;

impl Board {

    // https://official-stockfish.github.io/docs/nnue-pytorch-wiki/docs/nnue.html#basics
    // HalfKP NN input vector sparse encoding intuition, biasing NN for chess:
    // - King is clearly special and small changes cause "chaotic" effects to final eval.
    //   Thus for ONE side: 64 king buckets, only 1 bucket is used based on where the king is, all other buckets 0.
    // - Within the used bucket, 64 more buckets for each square, which expand to 12 neurons (6 piece types * 2 sides).
    //   Only 1 of the 12 is activated, 0 of 12 if no piece.
    // - Then expand king buckets by 2 for each side.
    // - XOR 56 part (flip Y perspective, but keep X, NOT a real life black side perspective, black sees A8 on bottom left) ->
    //   encoding not aware of black vs white, but side-to-move on top half of vector and opponent on bottom ->
    //   black/white can re-use each other's encodings if structurally the same, e.g. starting pos.
    //   - Same as showing black side phone image to mirror
    // - Metadata (castle, en passant) encoded into each side so it splits evenly between top half and bottom half of input vector.
    // - (Describe efficient inference aspects elsewhere.)
    pub const NNUE_PIECE_FEATURES: usize = 64 * 64 * 12;
    pub const NNUE_CASTLE_FEATURES: usize = 2;
    pub const NNUE_EP_FEATURES: usize = 8;
    pub const NNUE_HALF_SIZE: usize = Self::NNUE_PIECE_FEATURES + Self::NNUE_CASTLE_FEATURES + Self::NNUE_EP_FEATURES;
    pub const NNUE_TOTAL_SIZE: usize = 2 * Self::NNUE_HALF_SIZE;

    /// Computes active feature indices for one perspective's half of the NNUE input vector.
    /// This is the shared core logic used by both encode_nnue (reference/testing) and nnue_refresh (inference).
    /// `perspective`: which player's king determines the king bucket — NOT side-to-move.
    /// `en_passant_file`: Some(file) if this perspective is the side-to-move and EP is available.
    fn nnue_half_indices(&self, perspective: Player, en_passant_file: Option<u8>) -> Vec<usize> {
        let mut indices = Vec::with_capacity(32);
        let king_sq = self.get_player_state(perspective).king_location._lsb_to_index() as usize;
        let (king_sq_idx, flip_mask) = if perspective == Player::White {
            (king_sq, 0)
        } else {
            (king_sq ^ 56, 56)
        };

        for player in [Player::White, Player::Black] {
            let mut piece_locs = self.get_player_state(player).piece_locs;
            piece_locs.consume_loop_indices(|idx| {
                if let Square::Occupied(piece, _) = self.d[idx as usize] {
                    let piece_idx = Self::piece_to_nnue_index(piece, player, perspective);
                    // 56 = 111000. XOR inverts the 1 part, and keeps the 0 part.
                    let sq_idx = (idx as usize) ^ flip_mask;
                    let bucket = king_sq_idx * 64 * 12 + sq_idx * 12 + piece_idx;
                    indices.push(bucket);
                }
            });
        }

        let castle_offset = Self::NNUE_PIECE_FEATURES;
        let perspective_state = self.get_player_state(perspective);
        if perspective_state.moved_castle_piece[CastleType::Oo as usize] {
            indices.push(castle_offset + 0);
        }
        if perspective_state.moved_castle_piece[CastleType::Ooo as usize] {
            indices.push(castle_offset + 1);
        }

        if let Some(f) = en_passant_file {
            let ep_offset = Self::NNUE_PIECE_FEATURES + Self::NNUE_CASTLE_FEATURES;
            indices.push(ep_offset + f as usize);
        }

        indices
    }

    fn piece_to_nnue_index(piece: Piece, player: Player, perspective: Player) -> usize {
        let base = match piece {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        };
        (base + branchless_mask!(player != perspective, 6)) as usize
    }

    // TODO IMMEDIATE REVIEW ALL BELOW

    /// Refreshes one perspective's accumulator by computing all active feature indices
    /// and summing their corresponding weight rows.
    pub fn nnue_refresh(&mut self, perspective: Player) {
        let weights = match crate::engine::nnue_input_weights() {
            Some(w) => w,
            None => {
                self.nnue_acc[perspective as usize].fill(0.0);
                return;
            }
        };

        let ep_file = if perspective == self.player_with_turn && self.en_passant_extra_target.has_target() {
            Some(self.en_passant_extra_target.index % 8)
        } else {
            None
        };

        let indices = self.nnue_half_indices(perspective, ep_file);

        let acc = &mut self.nnue_acc[perspective as usize];
        acc.fill(0.0);

        for &idx in &indices {
            let row_offset = idx * NNUE_L1_SIZE;
            for j in 0..NNUE_L1_SIZE {
                acc[j] += weights[row_offset + j];
            }
        }
    }

    pub fn nnue_refresh_both(&mut self) {
        self.nnue_refresh(Player::White);
        self.nnue_refresh(Player::Black);
    }

    pub fn nnue_on_move(&mut self) {
        self.nnue_refresh_both();
    }

    pub fn nnue_on_revert(&mut self) {
        self.nnue_refresh_both();
    }

    /// Reference implementation: produces the full sparse input vector for training compatibility.
    /// Top half = side-to-move perspective, bottom half = opponent perspective.
    pub fn encode_nnue(&self, out: &mut [i8; Self::NNUE_TOTAL_SIZE]) {
        out.fill(0);

        let player_with_turn = self.get_player_with_turn();
        let ep_file = if self.en_passant_extra_target.has_target() {
            Some(self.en_passant_extra_target.index % 8)
        } else {
            None
        };

        let stm_indices = self.nnue_half_indices(player_with_turn, ep_file);
        for idx in stm_indices {
            out[idx] = 1;
        }

        let opp_indices = self.nnue_half_indices(player_with_turn.other_player(), None);
        for idx in opp_indices {
            out[Self::NNUE_HALF_SIZE + idx] = 1;
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_nnue_encoding_sanity() {
        let mut board = Board::with_kings_only();
        board.set_by_file_rank_test('e', 2, Square::Occupied(Piece::Pawn, Player::White));

        let mut out = [0i8; Board::NNUE_TOTAL_SIZE];
        board.encode_nnue(&mut out);

        let mut ones = 0;
        for &val in out.iter() {
            if val == 1 { ones += 1; }
            else { assert_eq!(val, 0); }
        }

        // Total ones should be 6:
        // 3 pieces (2 kings, 1 pawn) * 2 perspectives = 6
        assert_eq!(ones, 6);

        // Check the white pawn encoding

        // From white perspective
        let king_sq = FastCoord::from_coord(&file_rank_to_xy('e', 1)).0 as usize;
        let pawn_sq = FastCoord::from_coord(&file_rank_to_xy('e', 2)).0 as usize;
        let piece_idx = 0; // White pawn encoding from white perspective
        let bucket = king_sq * 64 * 12 + pawn_sq * 12 + piece_idx;
        assert_eq!(out[bucket], 1);

        // From black perspective
        let king_sq_black = king_sq; // In this black perspective, black e8 == white e1 encoding
        let pawn_sq_black = FastCoord::from_coord(&file_rank_to_xy('e', 7)).0 as usize;
        assert_eq!(pawn_sq_black, pawn_sq ^ 56);
        let piece_idx_black = 6; // White pawn encoding from black perspective
        let bucket_black = king_sq_black * 64 * 12 + pawn_sq_black * 12 + piece_idx_black;
        assert_eq!(out[Board::NNUE_HALF_SIZE + bucket_black], 1);
    }

    #[test]
    fn test_nnue_encoding_symmetry() {
        // Test that White's perspective of a position is identical to Black's perspective
        // of the same position with colors and ranks flipped.

        let mut board_white = Board::with_kings_only();
        board_white.set_by_file_rank_test('d', 2, Square::Occupied(Piece::Pawn, Player::White));
        // White to move

        let mut out_white = [0i8; Board::NNUE_TOTAL_SIZE];
        board_white.encode_nnue(&mut out_white);

        let mut board_black = Board::with_kings_only();
        board_black.set_by_file_rank_test('d', 7, Square::Occupied(Piece::Pawn, Player::Black));
        // Black to move
        board_black.handle_move_no_revert(&MoveWithEval(MoveDescription::SkipMove, 0));
        assert_eq!(board_black.get_player_with_turn(), Player::Black);

        let mut out_black = [0i8; Board::NNUE_TOTAL_SIZE];
        board_black.encode_nnue(&mut out_black);

        for i in 0..Board::NNUE_HALF_SIZE {
            assert_eq!(out_white[i], out_black[i], "Mismatch at index {} in side-to-move half", i);
        }
        for i in 0..Board::NNUE_HALF_SIZE {
            assert_eq!(out_white[Board::NNUE_HALF_SIZE + i], out_black[Board::NNUE_HALF_SIZE + i], "Mismatch at index {} in opponent half", i);
        }
    }

    #[test]
    fn test_nnue_half_indices_matches_encode_nnue() {
        let mut board = Board::with_kings_only();
        board.set_by_file_rank_test('e', 2, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('d', 7, Square::Occupied(Piece::Pawn, Player::Black));

        // encode_nnue gives the reference sparse vector
        let mut out = [0i8; Board::NNUE_TOTAL_SIZE];
        board.encode_nnue(&mut out);

        // nnue_half_indices should produce the same active indices
        let stm = board.get_player_with_turn();
        let ep_file = if board.en_passant_extra_target.has_target() {
            Some(board.en_passant_extra_target.index % 8)
        } else {
            None
        };

        let stm_indices = board.nnue_half_indices(stm, ep_file);
        for &idx in &stm_indices {
            assert_eq!(out[idx], 1, "STM index {} should be 1 in encode_nnue output", idx);
        }

        let opp_indices = board.nnue_half_indices(stm.other_player(), None);
        for &idx in &opp_indices {
            assert_eq!(out[Board::NNUE_HALF_SIZE + idx], 1, "OPP index {} (offset {}) should be 1 in encode_nnue output", idx, Board::NNUE_HALF_SIZE + idx);
        }

        // Verify counts match
        let stm_ones: usize = out[0..Board::NNUE_HALF_SIZE].iter().filter(|&&v| v == 1).count();
        let opp_ones: usize = out[Board::NNUE_HALF_SIZE..].iter().filter(|&&v| v == 1).count();
        assert_eq!(stm_indices.len(), stm_ones, "STM indices count mismatch");
        assert_eq!(opp_indices.len(), opp_ones, "OPP indices count mismatch");
    }
}
