use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};
use super::coords::*;
use super::entities::*;
use super::move_list::*;
use super::move_test::*;
use super::check_handler::*;
use super::push_moves_handler::*;
use super::super::*;

pub enum RevertableMove {
    // (_, old hash to revert to)
    NormalMove([BeforeSquare; 2], u64),
    Castle(CastleType),
    NoOp
}

#[derive(Clone)]
pub struct PlayerState {
    // TODO First thing to switch into bitboards
    pub piece_locs: HashSet<Coord>,
    /// Order: oo, ooo
    pub moved_castle_piece: [bool; 2]
}

impl PlayerState {
    fn new() -> Self {
        Self {
            piece_locs: HashSet::new(),
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
        board.hash = board.calculate_hash();
        board
    }

    #[inline]
    pub fn get_square_hash(i: usize, piece: Piece, player: Player) -> u64 {
        RANDOM_NUMBER_KEYS.squares[i * PER_SQUARE_LEN + (piece as usize) + (player as usize) * PIECE_LEN]
    }

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

    pub fn set_by_xy(&mut self, x: u8, y: u8, s: Square) {
        if let Square::Occupied(_, occupied_player) = self.get_by_xy(x, y) {
            let piece_list = &mut self.get_player_state_mut(*occupied_player).piece_locs;
            piece_list.remove(&Coord(x, y));
        }

        if let Square::Occupied(_, new_player) = s {
            let piece_list = &mut self.get_player_state_mut(new_player).piece_locs;
            piece_list.insert(Coord(x, y));
        }

        self.d[y as usize * 8 + x as usize] = s;
    }

    pub fn set(&mut self, file: char, rank: u8, s: Square) {
        let Coord(x, y) = file_rank_to_xy(file, rank);
        self.set_by_xy(x, y, s);
    }

    //////////////////////////////////////////////////
    // Moves

    /// No hash changes
    fn apply_before_after_sqs(&mut self, sqs: &[BeforeAfterSquare], apply_or_undo: bool) {
        if apply_or_undo {
            for BeforeAfterSquare(fast_coord, _, after) in sqs.iter() {
                let coord = fast_coord.to_coord();
                self.set_by_xy(coord.0, coord.1, *after);
            }
        } else {
            for BeforeAfterSquare(fast_coord, before, _) in sqs.iter() {
                let coord = fast_coord.to_coord();
                self.set_by_xy(coord.0, coord.1, *before);
            }
        }
    }

    /// Hash and square changes for castling
    fn apply_castle(&mut self, castle_type: CastleType, apply_or_undo: bool) {

        let curr_player = self.get_player_with_turn();
        let curr_player_state = self.get_player_state_mut(curr_player);

        let old_player = curr_player.other_player();
        let old_player_state = self.get_player_state_mut(old_player);

        let (player_num, expected_castle_bool) = if apply_or_undo {
            (curr_player as usize, false)
        } else {
            (old_player as usize, true)
        };

        let sqs: &[BeforeAfterSquare] = if castle_type == CastleType::Oo {
            &CASTLE_UTILS.oo_sqs[player_num]
        } else {
            &CASTLE_UTILS.ooo_sqs[player_num]
        };
        self.apply_before_after_sqs(sqs, apply_or_undo);

        let castle_type_num = castle_type as usize;
        let castle_key = RANDOM_NUMBER_KEYS.moved_castle_piece[castle_type_num][player_num];
        if curr_player_state.moved_castle_piece[castle_type_num] != expected_castle_bool {
            panic!("Illegal ooo state");
        } else {
            curr_player_state.moved_castle_piece[castle_type_num] = !expected_castle_bool;
            self.hash ^= castle_key;
        }
    }

    pub fn revert_move(&mut self, m: &RevertableMove) {
        match m {
            RevertableMove::NormalMove(snapshot, old_hash) => {
                for BeforeSquare(fast_coord, square) in snapshot.iter() {
                    let coord = fast_coord.to_coord();
                    self.set_by_xy(coord.0, coord.1, *square);
                }
                self.hash = *old_hash;
                self.player_with_turn = self.player_with_turn.other_player();
            },
            RevertableMove::Castle(castle_type) => {
                self.apply_castle(*castle_type, false);
                self.hash ^= RANDOM_NUMBER_KEYS.is_white_to_play;
                self.player_with_turn = self.player_with_turn.other_player();
            }
        }
    }

    /// Drags whatever is on the source square to the dest square, or applies a castle snapshot.
    /// Then updates hash, and switches turn.
    /// All correctness checks will be move generation's responsibility.
    pub fn handle_move(&mut self, m: &MoveWithEval) -> RevertableMove {
        let mut result: RevertableMove = RevertableMove::NoOp;

        match m.description() {
            MoveDescription::NormalMove(_from_coord, _to_coord) => {
                let old_hash = self.hash;

                let curr_player = self.get_player_with_turn();
                let curr_state = self.get_player_state(curr_player);

                let from_coord = _from_coord.to_coord();
                let to_coord = _to_coord.to_coord();

                let initial_from_sq = self.get_by_xy(from_coord.0, from_coord.1).clone();
                let initial_to_sq = self.get_by_xy(to_coord.0, to_coord.1).clone();

                self.set_by_xy(from_coord.0, from_coord.1, Square::Blank);
                self.set_by_xy(to_coord.0, to_coord.1, initial_from_sq);

                if let Square::Occupied(dragged_piece, dragged_piece_player) = initial_from_sq {

                    debug_assert!(dragged_piece_player == curr_player, "Tried to move for the wrong current player");

                    if dragged_piece == Piece::Rook {
                        curr_state.moved_castle_piece[0] = from_coord.0 == 7 && !curr_state.moved_castle_piece[0];
                        curr_state.moved_castle_piece[1] = from_coord.1 == 7 && !curr_state.moved_castle_piece[1];
                    } else if dragged_piece == Piece::King {
                        curr_state.moved_castle_piece[0] = !curr_state.moved_castle_piece[0];
                        curr_state.moved_castle_piece[1] = !curr_state.moved_castle_piece[1];
                    }

                    self.hash ^= Self::get_square_hash(_from_coord.value() as usize, dragged_piece, dragged_piece_player);

                    if let Square::Occupied(replaced_piece, replaced_piece_player) = initial_to_sq {
                        self.hash ^= Self::get_square_hash(_to_coord.value() as usize, replaced_piece, replaced_piece_player);
                    }
                    self.hash ^= Self::get_square_hash(_to_coord.value() as usize, dragged_piece, dragged_piece_player);
                } else {
                    panic!("Tried to move an empty square");
                }

                result = RevertableMove::NormalMove(
                    [BeforeSquare(*_from_coord, initial_from_sq), BeforeSquare(*_to_coord, initial_to_sq)], old_hash
                );
            }
            MoveDescription::Castle(castle_type) => {
                self.apply_castle(*castle_type, true);
                result = RevertableMove::Castle(*castle_type);
            }
        }

        self.hash ^= RANDOM_NUMBER_KEYS.is_white_to_play;
        self.player_with_turn = self.player_with_turn.other_player();

        result
    }

    fn can_castle(&mut self, king_traversal_coords: &[Coord], curr_player: Player) -> bool {
        let opponent = curr_player.other_player();

        let king_traversal_coords = king_traversal_coords;
        for Coord(x, y) in king_traversal_coords.iter() {
            self.set_by_xy(*x, *y, Square::Occupied(Piece::King, curr_player));
        }
        let can_castle = !is_checking(self, opponent);
        for Coord(x, y) in king_traversal_coords.iter() {
            self.set_by_xy(*x, *y, Square::Blank);
        }
        can_castle
    }

    /// Gets the final set of legal moves
    pub fn get_moves(&mut self, temp_moves: &mut MoveList, result: &mut MoveList) {

        let curr_player = self.get_player_with_turn();
        let opponent = curr_player.other_player();

        let mut moves_handler = PushToMoveListHandler { move_list: temp_moves };
        moves_handler.move_list.write_index = 0;

        fill_player(curr_player, false, self, &mut moves_handler);

        let mut check_handler = CheckDetectionHandler::new();
        for i in 0..moves_handler.move_list.write_index {
            let m = &moves_handler.move_list.v()[i];
            let revertable = self.handle_move(&m);

            check_handler.has_king_capture = false;
            fill_player(opponent, true, self, &mut check_handler); 

            self.revert_move(&revertable);
            if !check_handler.has_king_capture { result.write(m.clone()); }
        }

        let curr_state = self.get_player_state(curr_player);

        let can_castle = false;
        let wrote_can_castle = false;
        if !curr_state.moved_castle_piece[CastleType::Oo as usize] {
            can_castle = self.can_castle(&CASTLE_UTILS.oo_king_traversal_coords[curr_player as usize], curr_player);
            wrote_can_castle = true;
            if can_castle {
                result.write(MoveWithEval(MoveDescription::Castle(CastleType::Oo), 0.0));
            }
        }

        if !curr_state.moved_castle_piece[CastleType::Ooo as usize] {
            if !wrote_can_castle {
                can_castle = self.can_castle(&CASTLE_UTILS.ooo_king_traversal_coords[curr_player as usize], curr_player);
            }
            if can_castle {
                result.write(MoveWithEval(MoveDescription::Castle(CastleType::Ooo), 0.0));
            }
        }
    }

    //////////////////////////////////////////////////
    // Board setup

    fn set_uniform_row(&mut self, rank: u8, player: Player, piece: Piece) {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank, Square::Occupied(piece, player));
        }
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
        self.set_uniform_row(2, Player::White, Piece::Pawn);
        self.set_main_row(8, Player::Black);
        self.set_uniform_row(7, Player::Black, Piece::Pawn);
    }
}
