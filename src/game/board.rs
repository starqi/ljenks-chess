
use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};
use super::coords::*;
use super::entities::*;
use super::move_list::*;
use super::basic_move_test::*;
use super::super::*;

#[derive(Clone)]
pub struct PlayerState {
    // TODO First thing to switch into bitboards
    pub piece_locs: HashSet<Coord>,
    pub moved_oo_piece: bool,
    pub moved_ooo_piece: bool,
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
            player_with_turn: Player::White,
            player_state: [PlayerState::new(), PlayerState::new()]
        };
        board.set_standard_rows();
        board
    }

    pub fn as_number(&self) -> i128 {
        let mut h: i128 = 0;
        for sq in self.d.iter() {
            h *= 13;
            h += 6 + match sq {
                Square::Blank => 0,
                Square::Occupied(piece, player) => (*piece as i8 + 1) * (player.get_multiplier() as i8)
            } as i128;
        }
        h * self.get_player_with_turn().get_multiplier() as i128
    }

    //////////////////////////////////////////////////
    // Player state

    pub fn get_player_with_turn(&self) -> Player {
        self.player_with_turn
    }

    pub fn get_player_state(&self, player: Player) -> &PlayerState {
        &self.player_state[player as usize]
    }

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

    pub fn undo_move(&mut self, m: &MoveSnapshot) {
        for sq in m.iter() {
            if let Some((Coord(x, y), before_after)) = sq {
                self.set_by_xy(*x, *y, before_after.0);
            }
        }
        let player_state = self.get_player_state_mut(self.player_with_turn.get_other_player());
        Self::update_castle_state_based_on_move(m, player_state, false);
        self.player_with_turn = self.player_with_turn.get_other_player();
    }

    pub fn make_move(&mut self, m: &MoveSnapshot) {
        for sq in m.iter() {
            if let Some((Coord(x, y), before_after)) = sq {
                self.set_by_xy(*x, *y, before_after.1);
            }
        }
        let player_state = self.get_player_state_mut(self.player_with_turn);
        Self::update_castle_state_based_on_move(m, player_state, true);
        self.player_with_turn = self.player_with_turn.get_other_player();
    }

    fn update_castle_state_based_on_move(m: &MoveSnapshot, player_state: &mut PlayerState, b: bool) {
        match m.2 {
            MoveDescription::Ooo | MoveDescription::Oo => {
                player_state.castled_somewhere = b;
                player_state.moved_oo_piece = b;
                player_state.moved_ooo_piece = b;
            },
            _ => {
                match m.2 {
                    MoveDescription::Capture(true, _, _) | MoveDescription::Move(true, _, _) => {
                        player_state.moved_oo_piece = b;
                    },
                    _ => ()
                };
                match m.2 {
                    MoveDescription::Capture(_, true, _) | MoveDescription::Move(_, true, _) => {
                        player_state.moved_ooo_piece = b;
                    },
                    _ => ()
                };
            }
        };
    }

    /// Gets the final set of legal moves
    pub fn get_moves(&mut self, temp_moves: &mut MoveList, result: &mut MoveList) {

        let mut handler = PushToMoveListHandler { move_list: temp_moves };

        handler.move_list.write_index = 0;
        fill_player(self.player_with_turn, false, self, &mut handler);

        filter_check_threats(
            self,
            self.player_with_turn.get_other_player(), 
            handler.move_list,
            0,
            handler.move_list.write_index,
            result
        );

        let (moved_oo_piece, moved_ooo_piece) = {
            let ps = self.get_player_state(self.player_with_turn);
            (ps.moved_oo_piece, ps.moved_ooo_piece)
        };

        if !moved_oo_piece {
            self.try_push_castle(
                &CASTLE_UTILS.oo_king_traversal_sqs[self.player_with_turn as usize],
                &CASTLE_UTILS.oo_move_snapshots[self.player_with_turn as usize],
                self.player_with_turn,
                handler.move_list,
                result
            );
        }

        if !moved_ooo_piece {
            self.try_push_castle(
                &CASTLE_UTILS.ooo_king_traversal_sqs[self.player_with_turn as usize],
                &CASTLE_UTILS.ooo_move_snapshots[self.player_with_turn as usize],
                self.player_with_turn,
                handler.move_list,
                result
            );
        }
    }

    /// Only does piece checks, not state checks
    fn try_push_castle(
        &mut self,
        king_travel_squares: &[Coord],
        move_snapshot: &MoveSnapshot,
        player_with_turn: Player,
        temp_moves: &mut MoveList,
        result: &mut MoveList
    ) {
        for Coord(x, y) in king_travel_squares.iter() {
            if let Square::Occupied(_, _) = self.get_by_xy(*x, *y) {
                return;
            } 
        }

        for Coord(x, y) in king_travel_squares.iter() {
            self.set_by_xy(*x, *y, Square::Occupied(Piece::King, player_with_turn));
        }

        let mut handler = PushToMoveListHandler { move_list: temp_moves };

        handler.move_list.write_index = 0;
        fill_player(player_with_turn.get_other_player(), true, self, &mut handler);

        let can_castle = !has_king_capture_move(temp_moves, 0, temp_moves.write_index, player_with_turn);
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
