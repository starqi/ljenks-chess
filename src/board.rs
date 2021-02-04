// TODO King, en passante, promotion, castle, castle block
// TODO Rust review - closure types, references to closure types, lifetimes, '_, for loop iter, into_iter, slices, Ref being auto cast
// TODO Split modules, currently too much access between classes
// TODO File, rank conversion spam
// TODO Panic if not causeable by user input


use log::{debug, info, warn, error};
use std::iter::Iterator;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};

pub type Coord = (u8, u8);

pub type CoordEvalList = Vec<(Coord, Coord, i32)>;

pub struct MoveList {
    v: CoordEvalList,
    pub write_index: usize
}

/// Writers are expected to assume `write_index` is set already to the correct location
impl MoveList {

    pub fn new(capacity: usize) -> MoveList {
        MoveList {
            v: Vec::with_capacity(capacity),
            write_index: 0
        }
    }

    pub fn get_v(&self) -> &CoordEvalList {
        &self.v
    }

    pub fn write(&mut self, from: Coord, to: Coord, eval: i32) {
        self.grow_with_access(self.write_index);
        let item = &mut self.v[self.write_index];
        item.0 = from;
        item.1 = to;
        item.2 = eval;
        self.write_index += 1;
    }

    fn grow_with_access(&mut self, requested_index: usize) {
        if requested_index >= self.v.len() {
            for _ in 0..requested_index - self.v.len() + 1 {
                self.v.push(((0, 0), (0, 0), 0));
            }
        }
    }

    pub fn sort_subset(&mut self, start: usize, end_exclusive: usize) {

    }
}

fn xy_to_file_rank(x: u8, y: u8) -> (char, u8) {
    (std::char::from_u32(x as u32 + ('a' as u32)).unwrap(), 8 - (y as u8))
}

pub fn xy_to_file_rank_safe(x: i32, y: i32) -> Result<(char, u8), Error> {
    if x < 0 || x > 7 || y < 0 || y > 7 {
        return Err(Error::XyOutOfBounds(x, y));
    }
    Ok(xy_to_file_rank(x as u8, y as u8))
}

fn file_rank_to_xy(file: char, rank: u8) -> Coord {
    let x = file as u32 - 'a' as u32;
    let y = 8 - rank;
    (x as u8, y)
}

// Checks are for public interface
pub fn file_rank_to_xy_safe(file: char, rank: u8) -> Result<Coord, Error> {
    if rank < 1 || rank > 8 {
        return Err(Error::RankOutOfBounds(rank));
    }
    let file_u32 = file as u32;
    if file_u32 < 'a' as u32 || file_u32 > 'h' as u32 {
        return Err(Error::FileOutOfBounds(file));
    }
    return Ok(file_rank_to_xy(file, rank));
}

#[derive(Copy, Clone, PartialEq)]
pub enum Piece {
    Pawn, Rook, Knight, Bishop, Queen, King
}

impl Piece {
    fn custom_fmt(&self, f: &mut Formatter<'_>, is_lower: bool) -> Result<(), fmt::Error> {
        let s = match self {
            Piece::Pawn => "P",
            Piece::Rook => "R",
            Piece::Knight => "N",
            Piece::Bishop => "B",
            Piece::Queen => "Q",
            Piece::King => "K"
        };

        if is_lower {
            write!(f, "{}", s.chars().nth(0).unwrap().to_lowercase())
        } else {
            write!(f, "{}", s)
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        self.custom_fmt(f, true)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player { 
    Black, White
}

impl Player {
    pub fn get_other_player(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black
        }
    }
}

#[derive(Copy, Clone)]
pub enum Square {
    Occupied(Piece, Player), Blank
}

impl Default for Square {
    fn default() -> Self {
        Square::Blank
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Square::Blank => {
                write!(f, ". ")
            },
            Square::Occupied(piece, player) => {
                let r = piece.custom_fmt(f, *player == Player::Black);
                write!(f, " ");
                r
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    RankOutOfBounds(u8),
    FileOutOfBounds(char),
    XyOutOfBounds(i32, i32)
}

// TODO Merge
#[derive(Default)]
struct OldMoveSquares {
    old_square_a: (Coord, Square),
    old_square_b: (Coord, Square)
}

pub struct RevertableMove {
    old_move_squares: OldMoveSquares,
    old_player: Player
}

// TODO Should be able to substitute bitboards for the same interface
// TODO Should also be able to generate bitboards with this class
pub struct MoveTest<'a> {
    src_x: i8,
    src_y: i8,
    src_piece: Piece,
    src_player: Player,
    data: &'a Board,
    can_capture_king: bool
}

impl MoveTest<'_> {

    pub fn fill_src(
        src_x: u8,
        src_y: u8,
        src_piece: Piece,
        src_player: Player,
        data: &Board,
        can_capture_king: bool,
        result: &mut MoveList
    ) {
        debug_assert!(src_x >= 0 && src_y >= 0 && src_x <= 7 && src_y <= 7);
        let t = MoveTest {
            src_x: src_x as i8, src_y: src_y as i8, src_piece, src_player, data, can_capture_king
        };
        // TODO Array
        match src_piece {
            Piece::Pawn => t.push_pawn(result),
            Piece::Queen => t.push_queen(result),
            Piece::Knight => t.push_knight(result),
            Piece::King => t.push_king(result),
            Piece::Bishop => t.push_bishop(result),
            Piece::Rook => t.push_rook(result)
        }
    }

    pub fn fill_player(
        player_with_turn: Player,
        data: &Board,
        can_capture_king: bool,
        result: &mut MoveList
    ) {
        for (x, y) in &data.get_player_state(player_with_turn).piece_locs {
            if let Square::Occupied(piece, player) = data._get_by_xy(*x, *y) {
                debug_assert!(player == player_with_turn);
                MoveTest::fill_src(*x, *y, piece, player, data, can_capture_king, result);
            } else {
                panic!("Empty square in {:?} player's piece locs", player_with_turn);
            }
        }
    }

    pub fn filter_check_threats(
        real_board: &mut Board,
        checking_player: Player,

        candidates_and_buf: &mut MoveList,
        candidates_start: usize,
        candidates_end_exclusive: usize,

        result: &mut MoveList
    ) {
        for i in candidates_start..candidates_end_exclusive {
            let revertable = real_board.get_revertable_move(candidates_and_buf, i);
            real_board.make_move(candidates_and_buf, i);

            candidates_and_buf.write_index = candidates_end_exclusive;
            MoveTest::fill_player(
                checking_player, 
                real_board,
                true,
                candidates_and_buf
            ); 
            let cand_write_end_exclusive = candidates_and_buf.write_index;

            let mut is_king_safe = true;
            for j in candidates_end_exclusive..cand_write_end_exclusive {
                let (_, (dest_x, dest_y), _) = candidates_and_buf.get_v()[j];
                if let Square::Occupied(piece, _) = real_board._get_by_xy(dest_x, dest_y) {
                    if piece == Piece::King {
                        is_king_safe = false;
                        break;
                    }
                }
            }
            if is_king_safe {
                let safe_move = candidates_and_buf.get_v()[i];
                result.write(safe_move.0, safe_move.1, safe_move.2);
            }
            real_board.revert_move(&revertable);
        }
    }

    fn push(&self, test_dest_x: i8, test_dest_y: i8, capture: bool, result: &mut MoveList) -> bool {
        if test_dest_x < 0 || test_dest_x > 7 || test_dest_y < 0 || test_dest_y > 7 {
            return true;
        }

        let (dest_x, dest_y) = (test_dest_x as u8, test_dest_y as u8);

        let (moveable, terminate) = match self.data._get_by_xy(dest_x, dest_y) {
            Square::Occupied(dest_piece, dest_square_player) => {(
                capture && dest_square_player != self.src_player && (self.can_capture_king || dest_piece != Piece::King), 
                    true
            )},
            Square::Blank => {
                (true, false)
            }
        };

        debug!("{},{} moveable={} terminate={}", dest_x, dest_y, moveable, terminate);

        if moveable {
            result.write((self.src_x as u8, self.src_y as u8), (dest_x, dest_y), 0);
        }

        return terminate;
    }

    fn push_pawn(&self, result: &mut MoveList) {
        let (y_delta, jump_row) = match self.src_player {
            Player::Black => (1, 1),
            Player::White => (-1, 6)
        };

        let (x, y) = (self.src_x as i8, self.src_y as i8);
        if !self.push(x, y + y_delta, false, result) {
            if y == jump_row {
                self.push(x, y + y_delta * 2, false, result);
            }
        }

        for x_delta in -1..=1 {
            if x_delta == 0 { continue; }

            let x_p_delta: i8 = x + x_delta;
            let y_p_delta: i8 = y + y_delta;

            if x_p_delta < 0 || x_p_delta > 7 { continue; }
            if y_p_delta < 0 || y_p_delta > 7 { continue; }

            if let Square::Occupied(_, angled_player) = self.data._get_by_xy(x_p_delta as u8, y_p_delta as u8) {
                if angled_player != self.src_player {
                    self.push(x + x_delta, y + y_delta, true, result);
                }
            }
        }

    }

    fn push_rook(&self, result: &mut MoveList) {
        for _i in 1..=self.src_x {
            let i = self.src_x - _i;
            if self.push(i, self.src_y, true, result) { break; }
        }
        for i in self.src_x + 1..=7 {
            if self.push(i, self.src_y, true, result) { break; }
        }
        for _i in 1..=self.src_y {
            let i = self.src_y - _i;
            if self.push(self.src_x, i, true, result) { break; }
        }
        for i in self.src_y + 1..=7 {
            if self.push(self.src_x, i, true, result) { break; }
        }
    }

    fn push_bishop(&self, result: &mut MoveList) {

        for i in 1..=self.src_x {
            if self.push(self.src_x - i, self.src_y - i, true, result) { break; }
        }
        for i in 1..=self.src_x {
            if self.push(self.src_x - i, self.src_y + i, true, result) { break; }
        }
        for i in 1..=8 - (self.src_x + 1) {
            if self.push(self.src_x + i, self.src_y - i, true, result) { break; }
        }
        for i in 1..=8 - (self.src_x + 1) {
            if self.push(self.src_x + i, self.src_y + i, true, result) { break; }
        }
    }

    fn push_knight(&self, result: &mut MoveList) {

        self.push(self.src_x - 1, self.src_y + 2, true, result);
        self.push(self.src_x - 1, self.src_y - 2, true, result);

        self.push(self.src_x - 2, self.src_y + 1, true, result);
        self.push(self.src_x - 2, self.src_y - 1, true, result);

        self.push(self.src_x + 2, self.src_y + 1, true, result);
        self.push(self.src_x + 2, self.src_y - 1, true, result);

        self.push(self.src_x + 1, self.src_y + 2, true, result);
        self.push(self.src_x + 1, self.src_y - 2, true, result);
    }

    fn push_queen(&self, result: &mut MoveList) {
        self.push_bishop(result);
        self.push_rook(result);
    }

    fn push_king(&self, result: &mut MoveList) {
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 { continue; }
                self.push(self.src_x + i, self.src_y + j, true, result);
            }
        }
    }
}


pub struct PlayerState {
    pub piece_locs: HashSet<Coord>,
    pub did_castle: bool
}

impl PlayerState {
    fn new() -> PlayerState {
        PlayerState {
            piece_locs: HashSet::new(),
            did_castle: false
        }
    }

    fn reset(&mut self) {
        self.piece_locs.clear();
        self.did_castle = false;
    }
}

pub struct Board {
    player_with_turn: Player,
    d: [Square; 64],
    black_state: PlayerState,
    white_state: PlayerState
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
    pub fn new() -> Board {
        let mut board = Board {
            d: [Square::Blank; 64],
            player_with_turn: Player::White,
            black_state: PlayerState::new(),
            white_state: PlayerState::new()
        };
        board.set_standard_rows();
        board
    }

    pub fn restart(&mut self) {
        self.black_state.reset();
        self.white_state.reset();
        self.d = [Square::Blank; 64];
        self.set_standard_rows();
    }

    pub fn import_from(&mut self, other: &Board) {
        &self.d[..].copy_from_slice(&other.d);
        self.player_with_turn = other.player_with_turn;

        // FIXME Clone interface, recursive
        self.black_state.did_castle = other.black_state.did_castle;
        self.black_state.piece_locs = other.black_state.piece_locs.clone();
        self.white_state.did_castle = other.white_state.did_castle;
        self.white_state.piece_locs = other.white_state.piece_locs.clone();
    }

    pub fn get_player_with_turn(&self) -> Player {
        self.player_with_turn
    }

    pub fn get_player_state(&self, player: Player) -> &PlayerState {
        match player {
            Player::White => &self.white_state,
            Player::Black => &self.black_state
        }
    }

    pub fn get(&self, file: char, rank: u8) -> Result<Square, Error> {
        let (x, y) = file_rank_to_xy_safe(file, rank)?;
        self.get_by_xy(x, y)
    }

    // FIXME Checks
    pub fn get_by_xy(&self, x: u8, y: u8) -> Result<Square, Error> {
        Ok(self._get_by_xy(x, y))
    }

    pub fn set(&mut self, file: char, rank: u8, s: Square) -> Result<(), Error> {
        let (x, y) = file_rank_to_xy_safe(file, rank)?;
        self.set_by_xy(x, y, s)?;
        Ok(())
    }

    // FIXME Checks
    pub fn set_by_xy(&mut self, x: u8, y: u8, s: Square) -> Result<(), Error> {
        self._set_by_xy(x, y, s);
        Ok(())
    }

    pub fn revert_move(&mut self, r: &RevertableMove) {
        self._revert_move(&r.old_move_squares);
        self.player_with_turn = r.old_player;
    }

    /// No restrictions, equivalent to moving a piece anywhere replacing anything on a real life board 
    pub fn make_move(&mut self, moves: &MoveList, index: usize) {

        let m = match moves.get_v().get(index) {
            None => panic!("Make move out of bounds {}/{}", index, moves.get_v().len()),
            Some(_m) => _m
        };

        let source = m.0;
        let dest = m.1;

        if let Ok(Square::Occupied(piece, player)) = self.get_by_xy(source.0, source.1) {

            /*
            let player_state = self._get_player_state(player);
            // TODO Unexpected behaviour if board not started in standard format
            // TODO Extract
            match player {
                Player::White => {
                    if !player_state.did_castle {
                        let rook_moved = piece == Piece::Rook && (
                            (source.0 == 7 && source.1 == 0) ||
                            (source.0 == 7 && source.1 == 7)
                        );
                        let king_moved = piece == Piece::King && (
                            (source.0 == 4 && source.1 == 7)
                        );
                        if rook_moved || king_moved {
                            player_state.did_castle = true;
                        }
                    }
                }
                Player::Black => {
                    if !player_state.did_castle {
                        let rook_moved = piece == Piece::Rook && (
                            (source.0 == 0 && source.1 == 0) ||
                            (source.0 == 0 && source.1 == 7)
                        );
                        let king_moved = piece == Piece::King && (
                            (source.0 == 4 && source.1 == 0)
                        );
                        if rook_moved || king_moved {
                            player_state.did_castle = true;
                        }
                    }

                }
            }
            */

            if piece == Piece::Pawn && ((dest.1 == 7 && player == Player::Black) || (dest.1 == 1 && player == Player::White)) {
                self._set_by_xy(dest.0, dest.1, Square::Occupied(Piece::Queen, player));
            } else {
                self._set_by_xy(dest.0, dest.1, Square::Occupied(piece, player));
            }
            self._set_by_xy(source.0, source.1, Square::Blank);
        } else {
            panic!("Move list origin square is blank");
        }

        self.player_with_turn = self.player_with_turn.get_other_player();
    }

    pub fn get_revertable_move(&self, moves: &MoveList, index: usize) -> RevertableMove {

        let m = match moves.get_v().get(index) {
            None => panic!("Get revertable move out of bounds {}/{}", index, moves.get_v().len()),
            Some(_m) => _m
        };

        let source = m.0;
        let dest = m.1;

        let old_square_a = self._get_by_xy(dest.0, dest.1);
        let old_square_b = self._get_by_xy(source.0, source.1);

        RevertableMove {
            old_move_squares: OldMoveSquares {
                old_square_a: ((dest.0, dest.1), old_square_a),
                old_square_b: ((source.0, source.1), old_square_b)
            },
            old_player: self.player_with_turn
        }
    }

    /// For the current player
    pub fn get_moves(&mut self, temp_moves: &mut MoveList, result: &mut MoveList) {
        temp_moves.write_index = 0;
        MoveTest::fill_player(self.get_player_with_turn(), self, false, temp_moves);
        MoveTest::filter_check_threats(
            self,
            self.get_player_with_turn().get_other_player(), 
            temp_moves,
            0,
            temp_moves.write_index,
            result
        );
    }

    fn _revert_move(&mut self, r: &OldMoveSquares) {

        let ((old_x1, old_y1), old_sqr1) = r.old_square_a;
        let ((old_x2, old_y2), old_sqr2) = r.old_square_b;

        self._set_by_xy(old_x1, old_y1, old_sqr1);
        self._set_by_xy(old_x2, old_y2, old_sqr2);
    }

    fn set_uniform_row(&mut self, rank: u8, player: Player, piece: Piece) -> Result<(), Error> {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank, Square::Occupied(piece, player))?;
        }
        Ok(())
    }

    fn set_main_row(&mut self, rank: u8, player: Player) -> Result<(), Error> {
        self.set('a', rank, Square::Occupied(Piece::Rook, player))?;
        self.set('b', rank, Square::Occupied(Piece::Knight, player))?;
        self.set('c', rank, Square::Occupied(Piece::Bishop, player))?;
        self.set('d', rank, Square::Occupied(Piece::Queen, player))?;
        self.set('e', rank, Square::Occupied(Piece::King, player))?;
        self.set('f', rank, Square::Occupied(Piece::Bishop, player))?;
        self.set('g', rank, Square::Occupied(Piece::Knight, player))?;
        self.set('h', rank, Square::Occupied(Piece::Rook, player))?;
        Ok(())
    }


    fn set_standard_rows(&mut self) {
        //self.set_main_row(1, Player::White).unwrap();
        //self.set_uniform_row(2, Player::White, Piece::Pawn).unwrap();
        //self.set_main_row(8, Player::Black).unwrap();
        //self.set_uniform_row(3, Player::Black, Piece::Pawn).unwrap();
        self.set('e', 1, Square::Occupied(Piece::King, Player::White)).unwrap();
        self.set('a', 7, Square::Occupied(Piece::Queen, Player::Black)).unwrap();
        self.set('b', 7, Square::Occupied(Piece::Queen, Player::Black)).unwrap();
        //self.set('a', 8, Square::Occupied(Piece::Queen, Player::Black)).unwrap();
        //self.set('h', 8, Square::Occupied(Piece::Queen, Player::Black)).unwrap();
        //self.set('d', 1, Square::Blank).unwrap();
        //self.set('a', 1, Square::Blank).unwrap();
        //self.set('h', 1, Square::Blank).unwrap();
    }

    fn _set_by_xy(&mut self, x: u8, y: u8, s: Square) {
        if let Ok(Square::Occupied(_, occupied_player)) = self.get_by_xy(x, y) {
            let piece_list = &mut self._get_player_state(occupied_player).piece_locs;
            piece_list.remove(&(x, y));
        }

        if let Square::Occupied(_, new_player) = s {
            let piece_list = &mut self._get_player_state(new_player).piece_locs;
            piece_list.insert((x, y));
        }

        self.d[y as usize * 8 + x as usize] = s;
    }

    fn _get_by_xy(&self, x: u8, y: u8) -> Square {
        return self.d[y as usize * 8 + x as usize];
    }

    fn _get_player_state(&mut self, player: Player) -> &mut PlayerState {
        match player {
            Player::White => &mut self.white_state,
            Player::Black => &mut self.black_state
        }
    }
}
