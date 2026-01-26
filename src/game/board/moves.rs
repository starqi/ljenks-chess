use super::*;

pub enum RevertableMove {
    // TODO IMMEDIATE Better style? Params are accessible outside module...

    /// (old squares, old hash to revert to, moved_castle_piece - first index is `Player` enum number, old king location)
    NormalMove([BeforeSquare; 2], u64, [[bool; 2]; 2], Bitboard, TargetSquare),

    /// (old squares, old hash to revert to, en passant target)
    DoublePawnJump([BeforeSquare; 2], u64, TargetSquare),
    EnPassant([BeforeSquare; 3], u64, TargetSquare),

    /// (oo/ooo, old hash to revert to, moved_castle_piece, old king location),
    Castle(CastleType, u64, [bool; 2], Bitboard, TargetSquare),
    NoOp(u64)
}

impl Board {
    pub fn revert_move(&mut self, m: &RevertableMove) {
        let opponent = self.get_player_with_turn().other_player();
        match m {
            RevertableMove::NormalMove(snapshot, old_hash, old_moved_castle_piece, old_king_location, old_en_passant_target) => {
                for BeforeSquare(fast_coord, square) in snapshot.iter() {
                    self.set_by_index_no_hash(fast_coord.0, *square);
                }

                self.get_player_state_mut(Player::White).moved_castle_piece = old_moved_castle_piece[Player::White as usize];
                self.get_player_state_mut(Player::Black).moved_castle_piece = old_moved_castle_piece[Player::Black as usize];

                let opponent_state = self.get_player_state_mut(opponent);
                opponent_state.king_location = *old_king_location;

                self.en_passant_extra_target = old_en_passant_target.clone();
                self.hash = *old_hash;
                // Reset en passant file to what it was before the move
                // This is handled by restoring the old hash, which includes the en passant state
            },
            RevertableMove::Castle(castle_type, old_hash, old_moved_castle_piece, old_king_location, old_en_passant_target) => {
                let sqs: &[BeforeAfterSquare] = if *castle_type == CastleType::Oo {
                    &CASTLE_UTILS.oo_sqs[opponent as usize]
                } else {
                    &CASTLE_UTILS.ooo_sqs[opponent as usize]
                };
                self.apply_before_after_sqs(sqs, false);

                let opponent_state = self.get_player_state_mut(opponent);
                opponent_state.moved_castle_piece = *old_moved_castle_piece;
                opponent_state.king_location = *old_king_location;
                opponent_state.is_castled = false;

                self.en_passant_extra_target = old_en_passant_target.clone();
                self.hash = *old_hash;
            }
            // TODO (Minor) Can't simplify, snapshots is len 2 and len 3 below?
            RevertableMove::DoublePawnJump(snapshot, old_hash, old_en_passant_target) => {
                for BeforeSquare(fast_coord, square) in snapshot.iter() {
                    self.set_by_index_no_hash(fast_coord.0, *square);
                }
                self.en_passant_extra_target = old_en_passant_target.clone();
                self.hash = *old_hash;
            },
            RevertableMove::EnPassant(snapshot, old_hash, old_en_passant_target) => {
                for BeforeSquare(fast_coord, square) in snapshot.iter() {
                    self.set_by_index_no_hash(fast_coord.0, *square);
                }
                self.en_passant_extra_target = old_en_passant_target.clone();
                self.hash = *old_hash;
            },
            RevertableMove::NoOp(old_hash) => {
                self.hash = *old_hash;
            }
        }
        self.player_with_turn = opponent;
    }

    pub fn is_capture(&self, m: &MoveWithEval) -> bool {
        if let MoveDescription::NormalMove(_, _to_coord, meta) = m.description() {
            if *meta == MoveMetadata::EnPassant { return true; }
            if let Square::Occupied(_, _) = self.get_by_index(_to_coord.value()) {
                return true;
            }
        }
        false
    }

    /// No hash changes
    fn apply_before_after_sqs(&mut self, sqs: &[BeforeAfterSquare], is_after: bool) {
        if is_after {
            for BeforeAfterSquare(fast_coord, _, after) in sqs.iter() {
                self.set_by_index(fast_coord.0, *after);
            }
        } else {
            for BeforeAfterSquare(fast_coord, before, _) in sqs.iter() {
                self.set_by_index(fast_coord.0, *before);
            }
        }
    }

    #[inline]
    fn update_castle_state_hash_for_piece(&mut self, player: Player, dragged_or_taken: Piece, origin_coord: &Coord) {
        self.update_castle_state_hash(
            player, 
            dragged_or_taken == Piece::King || (dragged_or_taken == Piece::Rook && origin_coord.0 == 7),
            dragged_or_taken == Piece::King || (dragged_or_taken == Piece::Rook && origin_coord.0 == 0)
        );
    }

    /// Only turns on, can't turn off, idempotent
    /// Some ridiculous branchless implementation
    fn update_castle_state_hash(&mut self, player: Player, moved_oo: bool, moved_ooo: bool) {
        let player_num = player as usize;
        let oo_key = RANDOM_NUMBER_KEYS.moved_castle_piece[0][player_num];
        let ooo_key = RANDOM_NUMBER_KEYS.moved_castle_piece[1][player_num];

        let c: [u64; 2] = [0, !0];

        self.hash ^= oo_key & c[self.get_player_state(player).moved_castle_piece[0] as usize];
        self.hash ^= ooo_key & c[self.get_player_state(player).moved_castle_piece[1] as usize];

        {
            let player_state = self.get_player_state_mut(player);
            player_state.moved_castle_piece[0] = player_state.moved_castle_piece[0] || moved_oo;
            player_state.moved_castle_piece[1] = player_state.moved_castle_piece[1] || moved_ooo;
        }

        self.hash ^= oo_key & c[self.get_player_state(player).moved_castle_piece[0] as usize];
        self.hash ^= ooo_key & c[self.get_player_state(player).moved_castle_piece[1] as usize];
    }

    fn set_en_passant_state_hash(&mut self, double_jump_target: &Coord) {
        self._clear_en_passant_state_hash();

        let player = self.get_player_with_turn();
        let delta: i8 = if player == Player::White { 1 } else { -1 };
        self.en_passant_extra_target.set(double_jump_target.0, ((double_jump_target.1 as i8) + delta) as u8);

        self.hash ^= Self::get_loc_hash_en_passant(&self.en_passant_extra_target);
    }

    #[inline]
    fn _clear_en_passant_state_hash(&mut self) {
        if self.en_passant_extra_target.has_target() {
            self.hash ^= Self::get_loc_hash_en_passant(&self.en_passant_extra_target);
            self.en_passant_extra_target.reset();
        }
    }

    fn clear_en_passant_state_hash(&mut self) {
        self._clear_en_passant_state_hash();
        self.en_passant_extra_target.reset();
    }

    #[inline]
    pub fn handle_move_no_revert(&mut self, m: &MoveWithEval) {
        let mut revertable_result = RevertableMove::NoOp(0);
        self.handle_move(m, &mut revertable_result);
    }

    /// All correctness checks will be move generation's responsibility.
    pub fn handle_move(
        &mut self,
        m: &MoveWithEval,
        // This board class will not be responsible for tracking the STACK of revertable moves during search, delegate to caller.
        // Therefore, the caller will also track which positions were reached for draw by repetition. 
        //
        // Also, use output variable to look more obviously like a copy elision optimization target.
        revertable_result: &mut RevertableMove
    ) {
        let old_hash = self.hash;
        match m.description() {
            MoveDescription::NormalMove(_from_coord, _to_coord, metadata) => {

                let from_sq_copy = *self.get_by_index(_from_coord.value());
                let to_sq_copy = *self.get_by_index(_to_coord.value());

                // Determine the result type based on metadata
                let result = if let Square::Occupied(_, _) = from_sq_copy {
                    match *metadata {
                        MoveMetadata::DoublePawnJump => {
                            *revertable_result = RevertableMove::DoublePawnJump(
                                [BeforeSquare(*_from_coord, from_sq_copy), BeforeSquare(*_to_coord, to_sq_copy)], 
                                old_hash,
                                self.en_passant_extra_target.clone()
                            )
                        },
                        MoveMetadata::EnPassant => {
                            let captured_pawn_coord = FastCoord::from_xy(_to_coord.get_x(), _from_coord.get_y());
                            let captured_pawn_square = BeforeSquare(captured_pawn_coord, *self.get_by_fast_coord(captured_pawn_coord));
                            *revertable_result = RevertableMove::EnPassant(
                                [BeforeSquare(*_from_coord, from_sq_copy), BeforeSquare(*_to_coord, to_sq_copy), captured_pawn_square], 
                                old_hash,
                                self.en_passant_extra_target.clone()
                            )
                        },
                        _ => {
                            let curr_player_state = self.get_player_state(self.get_player_with_turn());
                            *revertable_result = RevertableMove::NormalMove(
                                [BeforeSquare(*_from_coord, from_sq_copy), BeforeSquare(*_to_coord, to_sq_copy)], 
                                old_hash,
                                [self.get_player_state(Player::White).moved_castle_piece, self.get_player_state(Player::Black).moved_castle_piece],
                                curr_player_state.king_location,
                                self.en_passant_extra_target.clone()
                            )
                        }
                    }
                } else {
                    panic!("Unexpected move from empty square");
                };

                if let Square::Occupied(dragged_piece, dragged_piece_player) = from_sq_copy {
                    let curr_player = self.get_player_with_turn();
                    assert!(dragged_piece_player == curr_player, "Tried to move for the wrong current player");
                    let opponent = dragged_piece_player.other_player();

                    let from_coord = _from_coord.to_coord();
                    let to_coord = _to_coord.to_coord();

                    if let Square::Occupied(target_piece, target_piece_player) = to_sq_copy {
                        assert!(target_piece_player == opponent, "Unexpected wrong target piece player");
                        self.update_castle_state_hash_for_piece(opponent, target_piece, &to_coord);
                    }
                    self.update_castle_state_hash_for_piece(curr_player, dragged_piece, &from_coord);

                    {
                        self.set_by_index(_from_coord.0, Square::Blank);
                        if dragged_piece == Piece::Pawn && to_coord.1 == dragged_piece_player.last_row() {
                            // TODO (Promoting to queen) Add a method which configures preferred piece
                            self.set_by_index(_to_coord.0, Square::Occupied(Piece::Queen, dragged_piece_player));
                        } else {
                            self.set_by_index(_to_coord.0, from_sq_copy);
                            if *metadata == MoveMetadata::EnPassant {
                                let captured_pawn_coord = FastCoord::from_xy(_to_coord.get_x(), _from_coord.get_y());
                                self.set_by_index(captured_pawn_coord.0, Square::Blank);
                            }
                        }

                        if dragged_piece == Piece::King {
                            self.get_player_state_mut(curr_player).king_location = Bitboard::from_index(_to_coord.0);
                        }
                    }

                    if *metadata == MoveMetadata::DoublePawnJump {
                        self.set_en_passant_state_hash(&to_coord);
                    } else {
                        self.clear_en_passant_state_hash();
                    }
                } else {
                    console_error!("{}", self);
                    console_error!("{} {}", _from_coord, _to_coord);
                    panic!("Tried to move an empty square");
                }

                result
            }
            MoveDescription::Castle(castle_type) => {

                let curr_player = self.get_player_with_turn();
                let curr_player_num = curr_player as usize;
                let result = {
                    let curr_player_state = self.get_player_state(curr_player);
                    *revertable_result = RevertableMove::Castle(
                        *castle_type,
                        old_hash,
                        curr_player_state.moved_castle_piece,
                        curr_player_state.king_location,
                        self.en_passant_extra_target.clone()
                    )
                };

                let sqs: &[BeforeAfterSquare];
                if *castle_type == CastleType::Oo {
                    sqs = &CASTLE_UTILS.oo_sqs[curr_player_num];
                } else {
                    sqs = &CASTLE_UTILS.ooo_sqs[curr_player_num];
                }
                self.apply_before_after_sqs(sqs, true);
                // We moved the king, so we moved a castle piece for both castles, set both flags
                self.update_castle_state_hash(curr_player, true, true);

                let curr_state = self.get_player_state_mut(curr_player);
                curr_state.is_castled = true; // Does not need to be part of hash, but is useful to AI
                curr_state.king_location = Bitboard::from_index(CASTLE_UTILS.post_castle_king_sq[*castle_type as usize][curr_player_num].0);

                result
            }
            _ => {
                *revertable_result = RevertableMove::NoOp(old_hash)
            }
        };

        self.hash ^= RANDOM_NUMBER_KEYS.is_white_to_play;
        self.player_with_turn = self.player_with_turn.other_player();
    }

    /// Does not check if a castle piece has moved
    fn _can_castle(&mut self, blank_coords: &[FastCoord], king_traversal_coords: &[FastCoord], curr_player: Player) -> bool {
        let opponent = curr_player.other_player();

        for FastCoord(index) in blank_coords.iter() {
            if let Square::Occupied(_, _) = self.get_by_index(*index) {
                return false;
            }
        }

        let old_king_loc = {
            let curr_state = self.get_player_state_mut(curr_player);
            let _old_king_loc = curr_state.king_location;
            for FastCoord(index) in king_traversal_coords.iter() {
                curr_state.king_location.set_index(*index);
            }
            _old_king_loc
        };
        let can_castle = !self.is_checking(opponent);
        self.get_player_state_mut(curr_player).king_location = old_king_loc;
        can_castle
    }

    fn try_write_castle(&mut self, curr_player: Player, castle_type: CastleType, move_list: &mut MoveList) {
        if !self.get_player_state(curr_player).moved_castle_piece[castle_type as usize] {
            let curr_player_num = curr_player as usize;

            let blank_coords: &[FastCoord] = if castle_type == CastleType::Oo {
                &CASTLE_UTILS.oo_blank_coords[curr_player_num]
            } else {
                &CASTLE_UTILS.ooo_blank_coords[curr_player_num]
            };

            if self._can_castle(blank_coords, &CASTLE_UTILS.king_traversal_coords[castle_type as usize][curr_player as usize], curr_player) {
                move_list.write(MoveWithEval(MoveDescription::Castle(castle_type), 0));
            }
        }
    }

    /// Get moves for the *current* player
    pub fn get_moves(&mut self, temp_moves: &mut MoveList, result: &mut MoveList) {

        let curr_player = self.get_player_with_turn();

        temp_moves.write_index = 0;
        self.get_pseudo_moves_for(curr_player, temp_moves);
 
        for i in 0..temp_moves.write_index {
            let m = &temp_moves.v()[i];
            let mut revertable = RevertableMove::NoOp(0);
            self.handle_move(m, &mut revertable);
            let is_checking = self.is_checking(self.get_player_with_turn());
            self.revert_move(&revertable);
            if !is_checking { result.write(m.clone()); }
        }

        self.try_write_castle(curr_player, CastleType::Oo, result);
        self.try_write_castle(curr_player, CastleType::Ooo, result);
    }

    // TODO Attack/check via castle is missing
    pub fn get_checks_captures_for(&mut self, player: Player, temp_moves: &mut MoveList, result: &mut MoveList) {

        let opponent = player.other_player();
        let curr_state = self.get_player_state(player);
        let opponent_state = self.get_player_state(opponent);

        let opponent_king_coord = FastCoord(opponent_state.king_location._lsb_to_index());
        let params = CheckCaptureParams {
            curr_player_piece_locs: &curr_state.piece_locs,
            opponent_piece_locs: &opponent_state.piece_locs,
            king_potential_rook_atks: _write_rook_moves(opponent_king_coord, &curr_state.piece_locs, &opponent_state.piece_locs),
            king_potential_bishop_atks: _write_bishop_moves(opponent_king_coord, &curr_state.piece_locs, &opponent_state.piece_locs),
            king_potential_knight_atks: _write_knight_moves(opponent_king_coord, &curr_state.piece_locs),
            king_potential_pawn_atks: BITBOARD_PRESETS.pawn_captures[opponent as usize][opponent_king_coord.0 as usize]
        };

        temp_moves.write_index = 0;
        let mut curr_piece_locs_clone = curr_state.piece_locs.clone();
        curr_piece_locs_clone.consume_loop_indices(|index| {
            self.get_checks_captures_at(FastCoord(index), &params, result);
        });
 
        for i in 0..temp_moves.write_index {
            let m = &temp_moves.v()[i];
            let mut revertable = RevertableMove::NoOp(0);
            self.handle_move(m, &mut revertable);
            let is_checking = self.is_checking(self.get_player_with_turn());
            self.revert_move(&revertable);
            if !is_checking { result.write(m.clone()); }
        }
    }

    /// Precondition: `origin` piece is `player`'s piece
    fn is_checking_at(&self, player: Player, origin: FastCoord) -> bool {
        let state = self.get_player_state(player);
        let opponent_state = self.get_player_state(player.other_player());

        match self.get_by_index(origin.0) {
            Square::Occupied(Piece::Pawn, Player::White) => {
                white_pawn_hits_king(origin, &state.piece_locs, &opponent_state.piece_locs, &opponent_state.king_location)
            }
            Square::Occupied(Piece::Pawn, Player::Black) => {
                black_pawn_hits_king(origin, &state.piece_locs, &opponent_state.piece_locs, &opponent_state.king_location)
            }
            Square::Occupied(Piece::Queen, _) => queen_hits_king(origin, &state.piece_locs, &opponent_state.piece_locs, &opponent_state.king_location),
            Square::Occupied(Piece::Knight, _) => knight_hits_king(origin, &state.piece_locs, &opponent_state.king_location),
            Square::Occupied(Piece::King, _) => king_hits_king(origin, &state.piece_locs, &opponent_state.king_location),
            Square::Occupied(Piece::Bishop, _) => bishop_hits_king(origin, &state.piece_locs, &opponent_state.piece_locs, &opponent_state.king_location),
            Square::Occupied(Piece::Rook, _) => rook_hits_king(origin, &state.piece_locs, &opponent_state.piece_locs, &opponent_state.king_location),
            Square::Blank => false
        }
    }

    pub fn is_checking(&self, player: Player) -> bool {
        let mut piece_locs_clone = self.get_player_state(player).piece_locs.clone();
        piece_locs_clone.consume_loop_indices2(|index| {
            self.is_checking_at(player, FastCoord(index))
        })
    }

    /// Builds up the `AttackFromBoards` for both players.
    /// Not used anymore for now...
    pub fn rewrite_af_boards_both_players(&self, result: &mut AttackFromBoards) {
        result.reset();
        let mut piece_locs_clone = self.get_player_state(Player::White).piece_locs.clone();
        piece_locs_clone.consume_loop_indices(|index| {
            self.update_af_board_at(FastCoord(index), Player::White, result);
        });
        piece_locs_clone = self.get_player_state(Player::Black).piece_locs.clone();
        piece_locs_clone.consume_loop_indices(|index| {
            self.update_af_board_at(FastCoord(index), Player::Black, result);
        });
    }

    /// Precondition: `origin` piece is `player`'s piece.
    /// Builds up the `AttackFromBoards` for one piece at `origin` owned by `player`.
    /// Not used anymore for now...
    fn update_af_board_at(&self, origin: FastCoord, player: Player, result: &mut AttackFromBoards) {
        let curr_state = self.get_player_state(player);
        let opponent_state = self.get_player_state(player.other_player());

        match self.get_by_index(origin.0) {
            Square::Occupied(Piece::Pawn, Player::White) => {
                update_white_pawn_af(origin, &opponent_state.piece_locs, result);
            }
            Square::Occupied(Piece::Pawn, Player::Black) => {
                update_black_pawn_af(origin, &opponent_state.piece_locs, result);
            }
            Square::Occupied(Piece::Queen, _) => update_queen_af(origin, &curr_state.piece_locs, &opponent_state.piece_locs, result),
            Square::Occupied(Piece::Knight, _) => update_knight_af(origin, &curr_state.piece_locs, result),
            Square::Occupied(Piece::King, _) => update_king_af(origin, &curr_state.piece_locs, result),
            Square::Occupied(Piece::Bishop, _) => update_bishop_af(origin, &curr_state.piece_locs, &opponent_state.piece_locs, result),
            Square::Occupied(Piece::Rook, _) => update_rook_af(origin, &curr_state.piece_locs, &opponent_state.piece_locs, result),
            Square::Blank => {}
        };
    }

    pub fn get_pseudo_moves_at(&self, origin: FastCoord, result: &mut MoveList) {
        if let Square::Occupied(_, player) = self.get_by_index(origin.0) {
            self._get_pseudo_moves_at(origin, *player, result);
        }
    }

    /// Precondition: `origin` piece is `player`'s piece
    pub fn _get_pseudo_moves_at(&self, origin: FastCoord, player: Player, result: &mut MoveList) {
        let curr_state = self.get_player_state(player);
        let opponent_state = self.get_player_state(player.other_player());

        match self.get_by_index(origin.0) {
            Square::Occupied(Piece::Pawn, Player::White) => {
                write_white_pawn_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs, &self.en_passant_extra_target.bitboard);
            }
            Square::Occupied(Piece::Pawn, Player::Black) => {
                write_black_pawn_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs, &self.en_passant_extra_target.bitboard);
            }
            Square::Occupied(Piece::Queen, _) => write_queen_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs),
            Square::Occupied(Piece::Knight, _) => write_knight_moves(result, origin, &curr_state.piece_locs),
            Square::Occupied(Piece::King, _) => write_king_moves(result, origin, &curr_state.piece_locs),
            Square::Occupied(Piece::Bishop, _) => write_bishop_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs),
            Square::Occupied(Piece::Rook, _) => write_rook_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs),
            Square::Blank => {}
        };
    }

    /// Imaginary in that there is no piece there.
    pub fn get_imaginary_pseudo_move_at(&self, origin: FastCoord, piece: Piece, player: Player) -> Bitboard {
        let curr_state = self.get_player_state(player);
        let opponent_state = self.get_player_state(player.other_player());

        match piece {
            Piece::Pawn => {
                match player {
                    Player::White => _write_white_pawn_moves(origin, &curr_state.piece_locs, &opponent_state.piece_locs, &self.en_passant_extra_target.bitboard),
                    Player::Black => _write_black_pawn_moves(origin, &curr_state.piece_locs, &opponent_state.piece_locs, &self.en_passant_extra_target.bitboard)
                }
            },
            Piece::Queen => _write_queen_moves(origin, &curr_state.piece_locs, &opponent_state.piece_locs),
            Piece::Knight => _write_knight_moves(origin, &curr_state.piece_locs),
            Piece::King => _write_king_moves(origin, &curr_state.piece_locs),
            Piece::Bishop => _write_bishop_moves(origin, &curr_state.piece_locs, &opponent_state.piece_locs),
            Piece::Rook => _write_rook_moves(origin, &curr_state.piece_locs, &opponent_state.piece_locs)
        }
    }

    /// Precondition: `origin` piece is `params` current player's piece
    fn get_checks_captures_at(&self, origin: FastCoord, params: &CheckCaptureParams, result: &mut MoveList) {
        match self.get_by_index(origin.0) {
            Square::Occupied(Piece::Pawn, Player::White) => {
                write_white_pawn_ccs(result, origin, &params, &self.en_passant_extra_target.bitboard);
            }
            Square::Occupied(Piece::Pawn, Player::Black) => {
                write_black_pawn_ccs(result, origin, &params, &self.en_passant_extra_target.bitboard);
            }
            Square::Occupied(Piece::Queen, _) => write_queen_ccs(result, origin, params),
            Square::Occupied(Piece::Knight, _) => write_knight_ccs(result, origin, params),
            Square::Occupied(Piece::King, _) => write_king_captures(result, origin, params.curr_player_piece_locs, params.opponent_piece_locs),
            Square::Occupied(Piece::Bishop, _) => write_bishop_ccs(result, origin, params),
            Square::Occupied(Piece::Rook, _) => write_rook_ccs(result, origin, params),
            Square::Blank => {}
        };
    }

    // ["Pseudo" Moves]
    // Currently defined as not castling, and doesn't take into account illegally exposing king for capture.
    // Just how the logic is currently split.
    fn get_pseudo_moves_for(&self, player: Player, result: &mut MoveList) {
        let mut piece_locs_clone = self.get_player_state(player).piece_locs.clone();
        piece_locs_clone.consume_loop_indices(|index| {
            self._get_pseudo_moves_at(FastCoord(index), player, result);
        });
    }
}
