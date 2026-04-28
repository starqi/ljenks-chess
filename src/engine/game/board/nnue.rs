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
    // - Metadata (castle, en passant) encoded into each half; each half tracks its sides castle rights 
    //   - En passant target only on top half, by definition
    //   - TODO Not tracking both sides is a flaw?
    // - If black were to go first, white-first starting position encoding is reuseable to get the eval 
    // - (Describe efficient inference aspects elsewhere.)
    pub const NNUE_PIECE_FEATURES: usize = 64 * 64 * 12;
    pub const NNUE_CASTLE_FEATURES: usize = 2;
    pub const NNUE_EP_FEATURES: usize = 8;
    pub const NNUE_HALF_SIZE: usize = Self::NNUE_PIECE_FEATURES + Self::NNUE_CASTLE_FEATURES + Self::NNUE_EP_FEATURES;
    pub const NNUE_TOTAL_SIZE: usize = 2 * Self::NNUE_HALF_SIZE;
    pub const NNUE_MAX_FEATURES_PER_HALF: usize = 34; // 32 pieces + 2 castle + 1 EP, 34 for alignment

    /// Computes active feature indices for one perspective's half of the NNUE input vector.
    /// `perspective`: which player's king determines the king bucket — NOT side-to-move.
    /// `en_passant_file`: Some(file) if this perspective is the side-to-move and EP is available.
    /// Returns count of indices written into `out[0..count]`.
    fn nnue_half_indices(&self, perspective: Player, en_passant_file: Option<u8>, out: &mut [usize; Self::NNUE_MAX_FEATURES_PER_HALF]) -> usize {
        let mut count = 0;
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
                    out[count] = king_sq_idx * 64 * 12 + sq_idx * 12 + piece_idx;
                    count += 1;
                }
            });
        }

        let castle_offset = Self::NNUE_PIECE_FEATURES;
        let perspective_state = self.get_player_state(perspective);
        if perspective_state.moved_castle_piece[CastleType::Oo as usize] {
            out[count] = castle_offset + 0;
            count += 1;
        }
        if perspective_state.moved_castle_piece[CastleType::Ooo as usize] {
            out[count] = castle_offset + 1;
            count += 1;
        }

        if let Some(f) = en_passant_file {
            out[count] = Self::NNUE_PIECE_FEATURES + Self::NNUE_CASTLE_FEATURES + f as usize;
            count += 1;
        }

        count
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

        let mut indices = [0usize; Self::NNUE_MAX_FEATURES_PER_HALF];
        let count = self.nnue_half_indices(perspective, ep_file, &mut indices);

        let acc = &mut self.nnue_acc[perspective as usize];
        acc.fill(0.0);

        for i in 0..count {
            let row_offset = indices[i] * NNUE_L1_OUTPUT_SIZE;
            for j in 0..NNUE_L1_OUTPUT_SIZE {
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
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_nnue_encoding_sanity() {
        let mut board = Board::with_kings_only();
        board.set_by_file_rank_test('e', 2, Square::Occupied(Piece::Pawn, Player::White));

        // 3 features, 2 kings, 1 pawn, no castle (since castle = whether we moved a castle piece), no EP

        let mut white_indices = [0usize; Board::NNUE_MAX_FEATURES_PER_HALF];
        let white_count = board.nnue_half_indices(Player::White, None, &mut white_indices);
        assert_eq!(white_count, 3);

        let mut black_indices = [0usize; Board::NNUE_MAX_FEATURES_PER_HALF];
        let black_count = board.nnue_half_indices(Player::Black, None, &mut black_indices);
        assert_eq!(black_count, 3);

        // Verify white pawn from white perspective
        let king_sq = FastCoord::from_coord(&file_rank_to_xy('e', 1)).0 as usize;
        let pawn_sq = FastCoord::from_coord(&file_rank_to_xy('e', 2)).0 as usize;
        let piece_idx = 0; // White pawn from white perspective
        let expected_bucket = king_sq * 64 * 12 + pawn_sq * 12 + piece_idx;
        assert!(white_indices[..white_count].contains(&expected_bucket), "White pawn not found in white indices");

        // Verify white pawn from black perspective
        // Use fake coords, visualize in the "^56" view

        // e2 white pawn in ^56 view
        let pawn_sq_black = FastCoord::from_coord(&file_rank_to_xy('e', 7)).0 as usize;
        // White pawn from black perspective, sees it as an enemy piece
        let piece_idx_black = 6; 
        // e8 black king in ^56 view
        let black_king_sq_black = FastCoord::from_coord(&file_rank_to_xy('e', 1)).0 as usize;
        let expected_bucket_black = black_king_sq_black * 64 * 12 + pawn_sq_black * 12 + piece_idx_black;
        assert!(black_indices[..black_count].contains(&expected_bucket_black), "White pawn not found in black indices");
    }

    #[test]
    fn test_nnue_encoding_symmetry() {
        // Test that White's perspective of a position is identical to Black's perspective
        // of the same position with colors and ranks flipped.

        let mut board_white = Board::with_kings_only();
        board_white.set_by_file_rank_test('d', 2, Square::Occupied(Piece::Pawn, Player::White));
        // White to move

        let mut board_black = Board::with_kings_only();
        board_black.set_by_file_rank_test('d', 7, Square::Occupied(Piece::Pawn, Player::Black));
        // Black to move
        board_black.handle_move_no_revert(&MoveWithEval(MoveDescription::SkipMove, 0));
        assert_eq!(board_black.get_player_with_turn(), Player::Black);

        let mut stm_white = [0usize; Board::NNUE_MAX_FEATURES_PER_HALF];
        let stm_white_count = board_white.nnue_half_indices(Player::White, None, &mut stm_white);

        let mut stm_black = [0usize; Board::NNUE_MAX_FEATURES_PER_HALF];
        let stm_black_count = board_black.nnue_half_indices(Player::Black, None, &mut stm_black);

        assert_eq!(stm_white_count, stm_black_count);
        let mut w: Vec<usize> = stm_white[..stm_white_count].to_vec();
        let mut b: Vec<usize> = stm_black[..stm_black_count].to_vec();
        w.sort();
        b.sort();
        assert_eq!(w, b, "STM half indices mismatch");

        // Same thing but for the not side-to-move "bottom" accumulator

        let mut opp_white = [0usize; Board::NNUE_MAX_FEATURES_PER_HALF];
        let opp_white_count = board_white.nnue_half_indices(Player::Black, None, &mut opp_white);

        let mut opp_black = [0usize; Board::NNUE_MAX_FEATURES_PER_HALF];
        let opp_black_count = board_black.nnue_half_indices(Player::White, None, &mut opp_black);

        assert_eq!(opp_white_count, opp_black_count);
        let mut ow: Vec<usize> = opp_white[..opp_white_count].to_vec();
        let mut ob: Vec<usize> = opp_black[..opp_black_count].to_vec();
        ow.sort();
        ob.sort();
        assert_eq!(ow, ob, "OPP half indices mismatch");
    }
}
