
use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};
use super::coords::*;
use super::entities::*;
use super::move_list::*;
use super::move_test::*;
use super::check_handler::*;
use super::push_moves_handler::*;
use super::super::*;

#[derive(Clone)]
pub struct PlayerState {

    // TODO First thing to switch into bitboards
    pub piece_locs: HashSet<Coord>,

    pub moved_oo_piece: bool,
    pub moved_ooo_piece: bool,

    /// Redundant
    pub castled_somewhere: bool
}

impl PlayerState {
    fn new() -> Self {
        Self {
            piece_locs: HashSet::new(),
            moved_oo_piece: false,
            moved_ooo_piece: false,
            castled_somewhere: false
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
        if ws.moved_oo_piece { h ^= RANDOM_NUMBER_KEYS.moved_oo_piece[0]; }
        if ws.moved_ooo_piece { h ^= RANDOM_NUMBER_KEYS.moved_ooo_piece[0]; }
        if bs.moved_oo_piece { h ^= RANDOM_NUMBER_KEYS.moved_oo_piece[1]; }
        if bs.moved_ooo_piece { h ^= RANDOM_NUMBER_KEYS.moved_ooo_piece[1]; }

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

    pub fn handle_move(&mut self, m: &MoveSnapshot, apply_or_undo: bool) {
        for sq_holder in m.iter() {
            if let Some((Coord(x, y), BeforeAfterSquares(before, after))) = sq_holder {
                self.set_by_xy(*x, *y, if apply_or_undo { *after } else { *before });

                if let Square::Occupied(before_piece, before_player) = before {
                    self.hash ^= Self::get_square_hash(*y as usize * 8 + *x as usize, *before_piece, *before_player);
                }
                if let Square::Occupied(after_piece, after_player) = after {
                    debug_assert!(apply_or_undo == (*after_player == self.get_player_with_turn()), "Applying move for wrong player - {}", m);
                    self.hash ^= Self::get_square_hash(*y as usize * 8 + *x as usize, *after_piece, *after_player);
                }
            }
        }

        {
            let mut hash = self.get_hash();
            let old_player = self.get_player_with_turn().get_other_player();
            let old_player_state = self.get_player_state_mut(old_player);

            let oo_key = RANDOM_NUMBER_KEYS.moved_oo_piece[old_player as usize];
            let ooo_key = RANDOM_NUMBER_KEYS.moved_ooo_piece[old_player as usize];

            // Undo hash state so that neither flags are true
            if old_player_state.moved_oo_piece { hash ^= oo_key; }
            if old_player_state.moved_ooo_piece { hash ^= ooo_key; }

            // Change the actual flags to whatever they should be
            match m.2 {
                MoveDescription::Ooo | MoveDescription::Oo => {
                    old_player_state.castled_somewhere = apply_or_undo;
                    old_player_state.moved_oo_piece = apply_or_undo;
                    old_player_state.moved_ooo_piece = apply_or_undo;
                },
                _ => {
                    match m.2 {
                        MoveDescription::Capture(true, _, _) | MoveDescription::Move(true, _, _) => {
                            old_player_state.moved_oo_piece = apply_or_undo;
                        },
                        _ => ()
                    };
                    match m.2 {
                        MoveDescription::Capture(_, true, _) | MoveDescription::Move(_, true, _) => {
                            old_player_state.moved_ooo_piece = apply_or_undo;
                        },
                        _ => ()
                    };
                }
            };

            // Sync hash state with actual flags 
            if old_player_state.moved_oo_piece { hash ^= oo_key; }
            if old_player_state.moved_ooo_piece { hash ^= ooo_key; }
            self.hash = hash;
        }

        self.hash ^= RANDOM_NUMBER_KEYS.is_white_to_play;
        self.player_with_turn = self.player_with_turn.get_other_player();
    }

    /// Gets the final set of legal moves
    pub fn get_moves(&mut self, temp_moves: &mut MoveList, result: &mut MoveList) {

        let opponent = self.get_player_with_turn().get_other_player();
        let mut moves_handler = PushToMoveListHandler { move_list: temp_moves };
        moves_handler.move_list.write_index = 0;

        fill_player(self.get_player_with_turn(), false, self, &mut moves_handler);

        let mut check_handler = CheckDetectionHandler::new();
        for i in 0..moves_handler.move_list.write_index {
            let m = &moves_handler.move_list.get_v()[i];
            self.handle_move(&m, true);

            check_handler.has_king_capture = false;
            fill_player(opponent, true, self, &mut check_handler); 

            self.handle_move(&m, false);
            if !check_handler.has_king_capture { result.write(m.clone()); }
        }

        let (moved_oo_piece, moved_ooo_piece) = {
            let ps = self.get_player_state(self.player_with_turn);
            (ps.moved_oo_piece, ps.moved_ooo_piece)
        };

        if !moved_oo_piece {
            self.try_push_castle(
                &CASTLE_UTILS.oo_king_traversal_sqs[self.player_with_turn as usize],
                &CASTLE_UTILS.oo_move_snapshots[self.player_with_turn as usize],
                self.player_with_turn,
                result
            );
        }

        if !moved_ooo_piece {
            self.try_push_castle(
                &CASTLE_UTILS.ooo_king_traversal_sqs[self.player_with_turn as usize],
                &CASTLE_UTILS.ooo_move_snapshots[self.player_with_turn as usize],
                self.player_with_turn,
                result
            );
        }
    }

    /// Only does piece checks, not state checks, ie. does it visually look like we can castle (but maybe the rook is not the original rook)
    fn try_push_castle(
        &mut self,
        king_travel_squares: &[Coord],
        move_snapshot: &MoveSnapshot,
        player_with_turn: Player,
        result: &mut MoveList
    ) {
        for sq_holder in move_snapshot.get_squares() {
            if let Some((Coord(x, y), BeforeAfterSquares(before_sq, _))) = sq_holder {
                if *self.get_by_xy(*x, *y) != *before_sq {
                    return;
                }
            }
        }

        for Coord(x, y) in king_travel_squares.iter() {
            self.set_by_xy(*x, *y, Square::Occupied(Piece::King, player_with_turn));
        }
        let can_castle = !is_checking(self, player_with_turn.get_other_player());
        for Coord(x, y) in king_travel_squares.iter() {
            self.set_by_xy(*x, *y, Square::Blank);
        }

        if can_castle {
            result.clone_and_write(move_snapshot);
        }
    }

    //////////////////////////////////////////////////

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
