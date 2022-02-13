use std::fmt::{Display, Formatter, self};
use super::coords::*;
use super::entities::*;
use super::move_list::*;
use super::move_test::*;
use super::super::*;
use super::bitboard::*;

pub enum RevertableMove {
    /// (old squares, old hash to revert to, moved_castle_piece - first index is `Player` enum number, old king location)
    NormalMove([BeforeSquare; 2], u64, [[bool; 2]; 2], Bitboard),
    /// (oo/ooo, old hash to revert to, moved_castle_piece, old king location),
    Castle(CastleType, u64, [bool; 2], Bitboard),
    NoOp(u64)
}

#[derive(Clone)]
pub struct PlayerState {
    pub piece_locs: Bitboard,
    pub king_location: Bitboard,
    /// Index: `CastleType` enum number
    pub moved_castle_piece: [bool; 2]
}

impl PlayerState {
    fn new() -> Self {
        Self {
            piece_locs: Bitboard(0),
            king_location: Bitboard(0),
            moved_castle_piece: [false, false]
        }
    }
}

#[derive(Clone)]
pub struct Board {
    player_with_turn: Player,
    d: [Square; 64],
    hash: u64,
    player_state: [PlayerState; 2]
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        for i in 0..self.d.len() {
            if i % 8 == 0 && i != 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}", self.d[i])?;
        }
        Ok(())
    }
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            d: [Square::Blank; 64],
            hash: 0,
            player_with_turn: Player::White,
            player_state: [PlayerState::new(), PlayerState::new()]
        };
        board.set_standard_rows();
        board.get_player_state_mut(Player::White).king_location = Bitboard::from_index(CASTLE_UTILS.pre_castle_king_sq[Player::White as usize].0);
        board.get_player_state_mut(Player::Black).king_location = Bitboard::from_index(CASTLE_UTILS.pre_castle_king_sq[Player::Black as usize].0);
        board.hash = board.calculate_hash();
        board
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        let mut board = Self {
            d: [Square::Blank; 64],
            hash: 0,
            player_with_turn: Player::White,
            player_state: [PlayerState::new(), PlayerState::new()]
        };
        board
    }

    pub fn stringify_move(&self, m: &MoveWithEval) -> String {
        match m.description() {
            MoveDescription::NormalMove(_from_coord, _to_coord) => {
                let square = self.get_by_index(_from_coord.value());
                // Since a piece should be on the after square,
                // the square will stringify to eg. k, K, p, P, then it becomes eg. Ke2
                format!("{}{} ({})", square, _to_coord, m.eval())
            },
            MoveDescription::Castle(castle_type) => {
                if *castle_type == CastleType::Oo {
                    format!("oo ({})", m.eval())
                } else {
                    format!("ooo ({})", m.eval())
                }
            },
            MoveDescription::SkipMove => {
                format!("skip ({})", m.eval())
            }
        }
    }

    pub fn print_move_list(&self, ml: &MoveList, start: usize, _end_exclusive: usize) {
        let end_exclusive = if _end_exclusive < ml.v().len() {
            _end_exclusive
        } else {
            ml.v().len()
        };

        console_log!("[Moves, {}-{}]", start, end_exclusive);
        for i in start..end_exclusive {
            console_log!("{}", self.stringify_move(&ml.v()[i]));
        }
        console_log!("");
    }

    #[inline]
    pub fn get_square_hash(i: usize, piece: Piece, player: Player) -> u64 {
        RANDOM_NUMBER_KEYS.squares[i * PER_SQUARE_LEN + (piece as usize) + (player as usize) * PIECE_LEN]
    }

    /// Slow hash calculation from scratch, currently just for assertions
    pub fn calculate_hash(&self) -> u64 {
        let mut h: u64 = 0;

        let mut i = 0usize;
        for sq in self.d.iter() {
            if let Square::Occupied(piece, player) = sq {
                h ^= Self::get_square_hash(i, *piece, *player);
            }
            i += 1;
        }

        let ws = self.get_player_state(Player::White);
        let bs = self.get_player_state(Player::Black);

        if ws.moved_castle_piece[CastleType::Oo as usize] {
            h ^= RANDOM_NUMBER_KEYS.moved_castle_piece[CastleType::Oo as usize][Player::White as usize]; 
        }
        if ws.moved_castle_piece[CastleType::Ooo as usize] {
            h ^= RANDOM_NUMBER_KEYS.moved_castle_piece[CastleType::Ooo as usize][Player::White as usize]; 
        }

        if bs.moved_castle_piece[CastleType::Oo as usize] {
            h ^= RANDOM_NUMBER_KEYS.moved_castle_piece[CastleType::Oo as usize][Player::Black as usize]; 
        }
        if bs.moved_castle_piece[CastleType::Ooo as usize] {
            h ^= RANDOM_NUMBER_KEYS.moved_castle_piece[CastleType::Ooo as usize][Player::Black as usize]; 
        } 

        if self.get_player_with_turn() == Player::White { h ^= RANDOM_NUMBER_KEYS.is_white_to_play; }
        
        h
    }

    #[inline]
    pub fn get_hash(&self) -> u64 {
        self.hash
    }

    pub fn assert_hash(&self) {
        assert_eq!(self.hash, self.calculate_hash());
    }

    //////////////////////////////////////////////////
    // Player state

    #[inline]
    pub fn get_player_with_turn(&self) -> Player {
        self.player_with_turn
    }

    #[inline]
    pub fn get_player_state(&self, player: Player) -> &PlayerState {
        &self.player_state[player as usize]
    }

    #[inline]
    fn get_player_state_mut(&mut self, player: Player) -> &mut PlayerState {
        &mut self.player_state[player as usize]
    }

    //////////////////////////////////////////////////
    // Get set squares

    pub fn get_safe(&self, file: char, rank: u8) -> Result<&Square, Error> {
        let Coord(x, y) = file_rank_to_xy_safe(file, rank)?;
        Ok(self.get_by_xy(x, y))
    }

    pub fn get_by_xy_safe(&self, x: i32, y: i32) -> Result<&Square, Error> {
        check_i32_xy(x, y)?;
        Ok(self.get_by_xy(x as u8, y as u8))
    }

    #[inline]
    pub fn get_by_xy(&self, x: u8, y: u8) -> &Square {
        return &self.d[y as usize * 8 + x as usize];
    }

    #[inline]
    pub fn get_by_index(&self, num: u8) -> &Square {
        return &self.d[num as usize];
    }

    #[cfg(test)]
    pub fn set_test(&mut self, file: char, rank: u8, s: Square) {
        self.set(file, rank, s);
    }

    fn set(&mut self, file: char, rank: u8, s: Square) {
        let Coord(x, y) = file_rank_to_xy(file, rank);
        self.set_by_xy(x, y, s);
    }

    #[inline]
    fn set_by_xy(&mut self, x: u8, y: u8, s: Square) {
        self.set_by_index(y * 8 + x, s);
    }

    fn set_by_index_no_hash(&mut self, index: u8, s: Square) {
        self.get_player_state_mut(Player::White).piece_locs.unset_index(index);
        self.get_player_state_mut(Player::Black).piece_locs.unset_index(index);
        if let Square::Occupied(_, new_player) = s {
            self.get_player_state_mut(new_player).piece_locs.set_index(index);
        }
        self.d[index as usize] = s;
    }

    pub fn set_by_index(&mut self, index: u8, s: Square) {
        self.get_player_state_mut(Player::White).piece_locs.unset_index(index);
        self.get_player_state_mut(Player::Black).piece_locs.unset_index(index);

        if let Square::Occupied(replaced_piece, replaced_piece_player) = self.get_by_index(index) {
            self.hash ^= Self::get_square_hash(index as usize, *replaced_piece, *replaced_piece_player);
        }
        if let Square::Occupied(new_piece, new_player) = s {
            self.hash ^= Self::get_square_hash(index as usize, new_piece, new_player);
            self.get_player_state_mut(new_player).piece_locs.set_index(index);
        }

        self.d[index as usize] = s;
    }

    //////////////////////////////////////////////////
    // Moves

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

    pub fn revert_move(&mut self, m: &RevertableMove) {
        let opponent = self.get_player_with_turn().other_player();
        match m {
            RevertableMove::NormalMove(snapshot, old_hash, old_moved_castle_piece, old_king_location) => {
                for BeforeSquare(fast_coord, square) in snapshot.iter() {
                    self.set_by_index_no_hash(fast_coord.0, *square);
                }

                self.get_player_state_mut(Player::White).moved_castle_piece = old_moved_castle_piece[Player::White as usize];
                self.get_player_state_mut(Player::Black).moved_castle_piece = old_moved_castle_piece[Player::Black as usize];

                let opponent_state = self.get_player_state_mut(opponent);
                opponent_state.king_location = *old_king_location;

                self.hash = *old_hash;
            },
            RevertableMove::Castle(castle_type, old_hash, old_moved_castle_piece, old_king_location) => {
                let sqs: &[BeforeAfterSquare] = if *castle_type == CastleType::Oo {
                    &CASTLE_UTILS.oo_sqs[opponent as usize]
                } else {
                    &CASTLE_UTILS.ooo_sqs[opponent as usize]
                };
                self.apply_before_after_sqs(sqs, false);

                let opponent_state = self.get_player_state_mut(opponent);
                opponent_state.moved_castle_piece = *old_moved_castle_piece;
                opponent_state.king_location = *old_king_location;

                self.hash = *old_hash;
            }
            RevertableMove::NoOp(old_hash) => {
                self.hash = *old_hash;
            }
        }
        self.player_with_turn = opponent;
    }

    pub fn is_capture(&self, m: &MoveWithEval) -> bool {
        if let MoveDescription::NormalMove(_, _to_coord) = m.description() {
            if let Square::Occupied(_, _) = self.get_by_index(_to_coord.value()) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn update_castle_for_piece(&mut self, player: Player, dragged_or_taken: Piece, origin_coord: &Coord) {
        self.update_castle_state_hash(
            player, 
            dragged_or_taken == Piece::King || (dragged_or_taken == Piece::Rook && origin_coord.0 == 7),
            dragged_or_taken == Piece::King || (dragged_or_taken == Piece::Rook && origin_coord.0 == 0)
        );
    }

    /// Only turns on, can't turn off
    fn update_castle_state_hash(&mut self, player: Player, moved_oo: bool, moved_ooo: bool) {
        let player_num = player as usize;
        let oo_key = RANDOM_NUMBER_KEYS.moved_castle_piece[0][player_num];
        let ooo_key = RANDOM_NUMBER_KEYS.moved_castle_piece[1][player_num];

        // Some stupid branchless experiment to unset and reset castle state and hash

        let c: [u64; 2] = [0, !0];

        self.hash ^= oo_key & c[self.get_player_state(player).moved_castle_piece[0] as usize];
        self.hash ^= ooo_key & c[self.get_player_state(player).moved_castle_piece[1] as usize];

        {
            let mut player_state = self.get_player_state_mut(player);
            player_state.moved_castle_piece[0] = player_state.moved_castle_piece[0] || moved_oo;
            player_state.moved_castle_piece[1] = player_state.moved_castle_piece[1] || moved_ooo;
        }

        self.hash ^= oo_key & c[self.get_player_state(player).moved_castle_piece[0] as usize];
        self.hash ^= ooo_key & c[self.get_player_state(player).moved_castle_piece[1] as usize];
    }

    /// All correctness checks will be move generation's responsibility.
    pub fn handle_move(&mut self, m: &MoveWithEval) -> RevertableMove {
        let old_hash = self.hash;
        let result = match m.description() {
            MoveDescription::NormalMove(_from_coord, _to_coord) => {

                let from_sq_copy = *self.get_by_index(_from_coord.value());
                let to_sq_copy = *self.get_by_index(_to_coord.value());

                let result = {
                    let curr_player_state = self.get_player_state(self.get_player_with_turn());
                    RevertableMove::NormalMove(
                        [BeforeSquare(*_from_coord, from_sq_copy), BeforeSquare(*_to_coord, to_sq_copy)], 
                        old_hash,
                        [self.get_player_state(Player::White).moved_castle_piece, self.get_player_state(Player::Black).moved_castle_piece],
                        curr_player_state.king_location
                    )
                };

                if let Square::Occupied(dragged_piece, dragged_piece_player) = from_sq_copy {
                    let curr_player = self.get_player_with_turn();
                    assert!(dragged_piece_player == curr_player, "Tried to move for the wrong current player");
                    let opponent = dragged_piece_player.other_player();

                    let from_coord = _from_coord.to_coord();
                    let to_coord = _to_coord.to_coord();

                    if let Square::Occupied(target_piece, target_piece_player) = to_sq_copy {
                        assert!(target_piece_player == opponent, "Unexpected wrong target piece player");
                        self.update_castle_for_piece(opponent, target_piece, &to_coord);
                    } 
                    self.update_castle_for_piece(curr_player, dragged_piece, &from_coord);

                    {
                        self.set_by_index(_from_coord.0, Square::Blank);
                        if dragged_piece == Piece::Pawn && to_coord.1 == dragged_piece_player.last_row() {
                            // TODO Add a method which configures preferred piece
                            self.set_by_index(_to_coord.0, Square::Occupied(Piece::Queen, dragged_piece_player));
                        } else {
                            self.set_by_index(_to_coord.0, from_sq_copy);
                        }

                        if dragged_piece == Piece::King {
                            self.get_player_state_mut(curr_player).king_location = Bitboard::from_index(_to_coord.0);
                        }
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
                    RevertableMove::Castle(*castle_type, old_hash, curr_player_state.moved_castle_piece, curr_player_state.king_location)
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
                self.get_player_state_mut(curr_player).king_location = Bitboard::from_index(CASTLE_UTILS.post_castle_king_sq[*castle_type as usize][curr_player_num].0);

                result
            }
            _ => {
                RevertableMove::NoOp(old_hash)
            }
        };

        self.hash ^= RANDOM_NUMBER_KEYS.is_white_to_play;
        self.player_with_turn = self.player_with_turn.other_player();

        result
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

    /// Get moves for the current player
    pub fn get_moves(&mut self, temp_moves: &mut MoveList, result: &mut MoveList) {

        let curr_player = self.get_player_with_turn();

        temp_moves.write_index = 0;
        self.get_pseudo_moves_for(curr_player, temp_moves);
 
        for i in 0..temp_moves.write_index {
            let m = &temp_moves.v()[i];
            let revertable = self.handle_move(m);
            let is_checking = self.is_checking(self.get_player_with_turn());
            self.revert_move(&revertable);
            if !is_checking { result.write(m.clone()); }
        }

        self.try_write_castle(curr_player, CastleType::Oo, result);
        self.try_write_castle(curr_player, CastleType::Ooo, result);
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

    pub fn rewrite_af_boards(&self, result: &mut AttackFromBoards) {
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

    /// Precondition: `origin` piece is `player`'s piece
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

    fn get_pseudo_moves_at(&self, origin: FastCoord, player: Player, result: &mut MoveList) {
        let curr_state = self.get_player_state(player);
        let opponent_state = self.get_player_state(player.other_player());

        match self.get_by_index(origin.0) {
            Square::Occupied(Piece::Pawn, Player::White) => {
                write_white_pawn_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs);
            }
            Square::Occupied(Piece::Pawn, Player::Black) => {
                write_black_pawn_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs);
            }
            Square::Occupied(Piece::Queen, _) => write_queen_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs),
            Square::Occupied(Piece::Knight, _) => write_knight_moves(result, origin, &curr_state.piece_locs),
            Square::Occupied(Piece::King, _) => write_king_moves(result, origin, &curr_state.piece_locs),
            Square::Occupied(Piece::Bishop, _) => write_bishop_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs),
            Square::Occupied(Piece::Rook, _) => write_rook_moves(result, origin, &curr_state.piece_locs, &opponent_state.piece_locs),
            Square::Blank => {}
        };
    }

    pub fn get_pseudo_moves_for(&self, player: Player, result: &mut MoveList) {
        let mut piece_locs_clone = self.get_player_state(player).piece_locs.clone();
        piece_locs_clone.consume_loop_indices(|index| {
            self.get_pseudo_moves_at(FastCoord(index), player, result);
        });
    }

    //////////////////////////////////////////////////
    // Board setup

    fn set_uniform_row(&mut self, rank: u8, sq: Square) {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank, sq);
        }
    }

    #[cfg(test)]
    pub fn set_uniform_row_test(&mut self, rank: u8, sq: Square) {
        self.set_uniform_row(rank, sq);
    }

    fn set_main_row(&mut self, rank: u8, player: Player) {
        self.set('a', rank, Square::Occupied(Piece::Rook, player));
        self.set('b', rank, Square::Occupied(Piece::Knight, player));
        self.set('c', rank, Square::Occupied(Piece::Bishop, player));
        self.set('d', rank, Square::Occupied(Piece::Queen, player));
        self.set('e', rank, Square::Occupied(Piece::King, player));
        self.set('f', rank, Square::Occupied(Piece::Bishop, player));
        self.set('g', rank, Square::Occupied(Piece::Knight, player));
        self.set('h', rank, Square::Occupied(Piece::Rook, player));
    }

    fn set_standard_rows(&mut self) {
        self.set_main_row(1, Player::White);
        self.set_uniform_row(2, Square::Occupied(Piece:: Pawn, Player::White));
        self.set_main_row(8, Player::Black);
        self.set_uniform_row(7, Square::Occupied(Piece::Pawn, Player::Black));
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[ignore]
    #[test]
    fn board_eyeball_test() {
        let mut board = Board::new();
        board.set_uniform_row(2, Square::Blank);
        board.set_uniform_row(7, Square::Blank);

        let mut ml = MoveList::new(100);
        board.get_pseudo_moves_at(FastCoord::from_xy(0, 0), Player::White, &mut ml);

        let mut b = Bitboard(0);
        for m in ml.v() {
            if let MoveDescription::NormalMove(_, _to) = m.description() {
                b.set_index(_to.0);
            }
        }
        println!("{}", b);
    }

    #[ignore]
    #[test]
    fn attacked_from_eyeball_test() {
        let mut board = Board::new();
        board.set_uniform_row(2, Square::Blank);
        board.set_uniform_row(7, Square::Blank);

        let mut af = AttackFromBoards::new();
        board.rewrite_af_boards(&mut af);
        for y in 0..8 {
            for x in 0..8 {
                println!("{},{}\n{}", x, y, af.data[y * 8 + x]);
            }
        }
    }
}
