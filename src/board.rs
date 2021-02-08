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

pub type CoordEvalList = Vec<(Coord, Coord, f32)>;

pub static SRC_COORD_1_OO: u8 = 254;
pub static SRC_COORD_1_OOO: u8 = SRC_COORD_1_OO + 1;
pub static SRC_COORD_1_PROMOTE: u8 = 253;

// First is white, second is black, *assumes value of `Player` enum*

static OO_PRE_BLANKS: [[Coord; 2]; 2] = [
    [(5, 7), (6, 7)],
    [(5, 0), (6, 0)]
];
static OOO_PRE_BLANKS: [[Coord; 3]; 2] = [
    [(1, 7), (2, 7), (3, 7)],
    [(1, 0), (2, 0), (3, 0)]
];
// FIXME Rename revertable move to board subset
// FIXME Delete OO types below here, replace with subset
// FIXME Replace make_move to simply apply the subset with "apply_subset"
// FIXME Fix "apply_subset" to detect special codes, just like make_move
static OO_POST_BLANKS: [[Coord; 2]; 2] = [
    [(4, 7), (7, 7)],
    [(4, 0), (7, 0)]
];
static OOO_POST_BLANKS: [[Coord; 2]; 2] = [
    [(0, 7), (4, 7)],
    [(0, 0), (4, 0)]
];
static OO_KING_POS: [Coord; 2] = [(6, 7), (6, 0)];
static OOO_KING_POS: [Coord; 2] = [(2, 7), (2, 0)];
static OO_ROOK_POS: [Coord; 2] = [(5, 7), (5, 0)];
static OOO_ROOK_POS: [Coord; 2] = [(3, 7), (3, 0)];

/// Pre blank, post blank, king pos, rook pos
/// Must be dynamic because blank square sizes are different at compile
type CastlePositions = [
    (
        [Vec<Coord>; 2],
        &'static[[Coord; 2]; 2],
        &'static[Coord; 2],
        &'static[Coord; 2]
    ); 2
];

static mut CASTLE_POSITIONS: Option<CastlePositions> = None;

pub fn init() {
    unsafe {
        if let None = CASTLE_POSITIONS {
            let oo_pre = [
                Vec::from(OO_PRE_BLANKS[0]),
                Vec::from(OO_PRE_BLANKS[1])
            ];
            let ooo_pre = [
                Vec::from(OOO_PRE_BLANKS[0]),
                Vec::from(OOO_PRE_BLANKS[1])
            ];

            CASTLE_POSITIONS = Some([
                (oo_pre, &OO_POST_BLANKS, &OO_KING_POS, &OO_ROOK_POS),
                (ooo_pre, &OOO_POST_BLANKS, &OOO_KING_POS, &OOO_ROOK_POS)
            ]);
        }
    }
}

fn get_castle_pos() -> &'static CastlePositions {
    unsafe {
        if let Some(ref x) = CASTLE_POSITIONS {
            x
        } else {
            panic!("Castle positions should be initialized");
        }
    }
}

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

    pub fn write(&mut self, from: Coord, to: Coord, eval: f32) {
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
                self.v.push(((0, 0), (0, 0), 0.));
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
    Pawn = 0, Rook, Knight, Bishop, Queen, King
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

/// This order will be assumed in arrays!  
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player { 
    White = 0, Black
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

pub struct RevertableMove {
    squares: [Option<(Coord, Square)>; 4],
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
            let revertable = real_board.make_revertable_move(candidates_and_buf, i);
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
            result.write((self.src_x as u8, self.src_y as u8), (dest_x, dest_y), 0.);
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

#[derive(Clone)]
pub struct PlayerState {
    // TODO First bitboard switch
    pub piece_locs: HashSet<Coord>,
    pub can_castle: [bool; 2]
}

impl PlayerState {
    fn new() -> PlayerState {
        PlayerState {
            piece_locs: HashSet::new(),
            can_castle: [true, true]
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
    pub fn new() -> Board {
        let mut board = Board {
            d: [Square::Blank; 64],
            player_with_turn: Player::White,
            player_state: [PlayerState::new(), PlayerState::new()]
        };
        board.set_standard_rows();
        board
    }

    pub fn get_player_with_turn(&self) -> Player {
        self.player_with_turn
    }

    pub fn get_player_state(&self, player: Player) -> &PlayerState {
        &self.player_state[player as usize]
    }

    pub fn get(&self, file: char, rank: u8) -> Result<Square, Error> {
        let (x, y) = file_rank_to_xy_safe(file, rank)?;
        self.get_by_xy(x, y)
    }

    // FIXME Checks
    pub fn get_by_xy(&self, x: u8, y: u8) -> Result<Square, Error> {
        Ok(self._get_by_xy(x, y))
    }

    fn set(&mut self, file: char, rank: u8, s: Square) {
        let (x, y) = file_rank_to_xy(file, rank);
        self.set_by_xy(x, y, s);
    }

    pub fn revert_move(&mut self, r: &RevertableMove) {
        for i in 0..4 {
            if let Some(((x, y), square)) = r.squares[i] {
                self.set_by_xy(x, y, square);
            }
        }
        self.player_with_turn = r.old_player;
    }

    /// No restrictions, equivalent to moving a piece anywhere replacing anything on a real life board.
    /// Handles special moves if match, undefined piece movement if illegal moves. 
    /// Then flips turns.
    pub fn make_move(&mut self, moves: &MoveList, index: usize) {
        let ((source_x, source_y), (dest_x, dest_y), _) = moves.v[index];

        let mut did_special_move = false;
        let player_index = self.player_with_turn as usize;
        if source_x == SRC_COORD_1_OO || source_x == SRC_COORD_1_OOO {

            let castle_type_index = (source_x - SRC_COORD_1_OO) as usize;
            let (ref _pre_blank, _post_blank, _king_pos, _rook_pos) = get_castle_pos()[castle_type_index];

            let (pre_blank, post_blank, king_pos, rook_pos) = (
                &_pre_blank[player_index],
                _post_blank[player_index],
                _king_pos[player_index],
                _rook_pos[player_index],
            );

            for (x, y) in pre_blank.iter() {
                self.set_by_xy(*x, *y, Square::Blank);
            }
            for (x, y) in post_blank.iter() {
                self.set_by_xy(*x, *y, Square::Blank);
            }
            self.set_by_xy(king_pos.0, king_pos.1, Square::Occupied(Piece::King, self.player_with_turn));
            self.set_by_xy(rook_pos.0, rook_pos.1, Square::Occupied(Piece::Rook, self.player_with_turn));
            self.player_state[player_index].can_castle[castle_type_index] = false;

            did_special_move = true;

        } else if let Ok(Square::Occupied(src_piece, src_player)) = self.get_by_xy(source_x, source_y) {

            if src_piece == Piece::Pawn && (
                (dest_y == 7 && src_player == Player::Black) || (dest_y == 0 && src_player == Player::White)
            ) {
                self.set_by_xy(dest_x, dest_y, Square::Occupied(Piece::Queen, src_player));
                self.set_by_xy(source_x, source_y, Square::Blank);
                did_special_move = true;
            }
        }

        if !did_special_move {
            if let Ok(Square::Occupied(src_piece, src_player)) = self.get_by_xy(source_x, source_y) {
                self.set_by_xy(dest_x, dest_y, Square::Occupied(src_piece, src_player));
                self.set_by_xy(source_x, source_y, Square::Blank);
            } else {
                panic!("Move list origin square is blank");
            }
        }

        self.player_with_turn = self.player_with_turn.get_other_player();
    }

    pub fn make_revertable_move(&self, moves: &MoveList, index: usize) -> RevertableMove {

        let ((source_x, source_y), (dest_x, dest_y), _) = moves.v[index];
        let old_square_a = self._get_by_xy(dest_x, dest_y);
        let old_square_b = self._get_by_xy(source_x, source_y);

        RevertableMove {
            old_move_squares: OldMoveSquares {
                old_square_a: ((dest_x, dest_y), old_square_a),
                old_square_b: ((source_x, source_y), old_square_b)
            },
            old_player: self.player_with_turn
        }
    }

    /// For the current player
    pub fn get_moves(&mut self, temp_moves: &mut MoveList, result: &mut MoveList) {
        temp_moves.write_index = 0;
        MoveTest::fill_player(self.player_with_turn, self, false, temp_moves);
        MoveTest::filter_check_threats(
            self,
            self.player_with_turn.get_other_player(), 
            temp_moves,
            0,
            temp_moves.write_index,
            result
        );

        // Rewriting temp moves from 0 now
        let can_castle = self.get_player_state(self.player_with_turn).can_castle;
        if can_castle[0] {
            let blank_squares = OO_PRE_BLANKS[self.player_with_turn as usize];
            self.push_castle(&blank_squares, self.player_with_turn, SRC_COORD_1_OO, temp_moves, result);
        }
        if can_castle[1] {
            let blank_squares = OOO_PRE_BLANKS[self.player_with_turn as usize];
            self.push_castle(&blank_squares, self.player_with_turn, SRC_COORD_1_OOO, temp_moves, result);
        }
    }

    /// Separate from the normal candidate move + check threat pattern
    fn push_castle(
        &mut self,
        blank_squares: &[Coord],
        player_with_turn: Player,
        castle_src_1_op_code: u8,
        temp_moves: &mut MoveList,
        result: &mut MoveList
    ) {
        for (x, y) in blank_squares.iter() {
            if let Square::Occupied(_, _) = self._get_by_xy(*x, *y) {
                return;
            } 
        }

        let mut can_castle = true;
        for (x, y) in blank_squares.iter() {
            self.set_by_xy(*x, *y, Square::Occupied(Piece::King, player_with_turn));
        }
        temp_moves.write_index = 0;
        MoveTest::fill_player(
            player_with_turn.get_other_player(), self, true, temp_moves
        );
        for i in 0..temp_moves.write_index {
            let (_, (dest_x, dest_y), _) = temp_moves.v[i];
            if let Square::Occupied(piece, player) = self._get_by_xy(dest_x, dest_y) {
                debug_assert!(player == player_with_turn);
                if piece == Piece::King {
                    can_castle = false;
                    break;
                }
            }
        }
        for (x, y) in blank_squares.iter() {
            self.set_by_xy(*x, *y, Square::Blank);
        }

        if can_castle {
            result.write((castle_src_1_op_code, 0), (0, 0), 0.);
        }
    }

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

    fn set_by_xy(&mut self, x: u8, y: u8, s: Square) {
        if let Ok(Square::Occupied(_, occupied_player)) = self.get_by_xy(x, y) {
            let piece_list = &mut self.get_player_state_mut(occupied_player).piece_locs;
            piece_list.remove(&(x, y));
        }

        if let Square::Occupied(_, new_player) = s {
            let piece_list = &mut self.get_player_state_mut(new_player).piece_locs;
            piece_list.insert((x, y));
        }

        self.d[y as usize * 8 + x as usize] = s;
    }

    pub fn _get_by_xy(&self, x: u8, y: u8) -> Square {
        return self.d[y as usize * 8 + x as usize];
    }

    fn get_player_state_mut(&mut self, player: Player) -> &mut PlayerState {
        &mut self.player_state[player as usize]
    }
}
