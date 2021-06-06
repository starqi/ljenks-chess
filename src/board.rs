// TODO Should be able to castle if king path is not blocked, currently cares about whole covered path
// TODO King, en passante, promotion, castle, castle block
// TODO Rust review - closure types, references to closure types, lifetimes, '_, for loop iter, into_iter, slices, Ref being auto cast
// TODO Split modules, currently too much access between classes
// TODO File, rank conversion spam
// TODO Panic if not causeable by user input

use std::rc::Rc;
use log::{debug, info, warn, error};
use std::iter::Iterator;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};

pub type Coord = (u8, u8);

pub struct MoveList {
    v: Vec<MoveSnapshot>,
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

    pub fn get_v(&self) -> &Vec<MoveSnapshot> {
        &self.v
    }

    pub fn write(&mut self, board_subset: &MoveSnapshot) {
        self.write_and_move(*board_subset);
    }

    pub fn write_and_move(&mut self, board_subset: MoveSnapshot) {
        self.grow_with_access(self.write_index);
        self.v[self.write_index] = board_subset;
        self.write_index += 1;
    }

    fn grow_with_access(&mut self, requested_index: usize) {
        if requested_index >= self.v.len() {
            for _ in 0..requested_index - self.v.len() + 1 {
                self.v.push([None; 5]);
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

#[derive(Default)]
pub struct BeforeAfterSquares { 
    before: Square,
    after: Square
}

pub type Eval = f32;

// Fairly small bounded size is useable for the most complex move which is castling
pub type MoveSnapshot = [Option<(Coord, BeforeAfterSquares, Eval)>; 5];

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

    fn get_first_row(self) -> u8 {
        return if self == Player::White {
            7
        } else {
            1
        };
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

// TODO Should be able to substitute bitboards for the same interface
// TODO Should also be able to generate bitboards with this class
pub struct BasicMoveTest<'a> {
    src_x: i8,
    src_y: i8,
    src_piece: Piece,
    src_player: Player,
    data: &'a Board,
    can_capture_king: bool
}

impl <'a> BasicMoveTest<'a> {

    /// Pushes to `result` all the "basic" moves of a source piece with the special option to be able to capture a king
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
        let t = BasicMoveTest {
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

    /// Calls `fill_src` for all pieces owned by a player 
    pub fn fill_player(
        player_with_turn: Player,
        data: &Board,
        can_capture_king: bool,
        result: &mut MoveList
    ) {
        for (x, y) in &data.get_player_state(player_with_turn).piece_locs {
            if let Square::Occupied(piece, player) = data._get_by_xy(*x, *y) {
                debug_assert!(player == player_with_turn);
                BasicMoveTest::fill_src(*x, *y, piece, player, data, can_capture_king, result);
            } else {
                panic!("Empty square in {:?} player's piece locs", player_with_turn);
            }
        }
    }

    fn has_king_capture_move(
        moves: &mut MoveList,
        start: usize,
        end_exclusive: usize,
        checked_player: Player
    ) -> bool {
        for j in start..end_exclusive {
            let modified_sqs = moves.get_v()[j];
            for sq in modified_sqs.iter() {
                if let Some((_, before_after_sqs, _)) = sq {
                    if let Square::Occupied(before_piece, before_player) = before_after_sqs.before {
                        if before_piece == Piece::King && before_player == checked_player {
                            if let Square::Occupied(_, after_player) = before_after_sqs.after {
                                if after_player != checked_player {
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        }
        return true;
    }

    /// Filters out input moves which cannot be made because king will be captured
    pub fn filter_check_threats(
        real_board: &mut Board,
        checking_player: Player,

        candidates_and_buf: &mut MoveList,
        candidates_start: usize,
        candidates_end_exclusive: usize,

        result: &mut MoveList
    ) {
        let checked_player = checking_player.get_other_player();

        for i in candidates_start..candidates_end_exclusive {
            real_board.make_move(candidates_and_buf, i);

            candidates_and_buf.write_index = candidates_end_exclusive;
            BasicMoveTest::fill_player(
                checking_player, 
                real_board,
                true,
                candidates_and_buf
            ); 
            let cand_write_end_exclusive = candidates_and_buf.write_index;

            if !BasicMoveTest::has_king_capture_move(candidates_and_buf, candidates_end_exclusive, cand_write_end_exclusive, checked_player) {
                let safe_move = candidates_and_buf.get_v()[i];
                result.write(&safe_move);
            }

            real_board.undo_move(candidates_and_buf, i);
        }
    }

    /// `can_capture` - eg. Pawn cannot capture forwards
    /// Returns whether a move was added, and when applicable, whether the search along the same line should be terminated
    fn push(&self, test_dest_x: i8, test_dest_y: i8, can_capture: bool, replacement_piece: Option<Piece>, result: &mut MoveList) -> (bool, bool) {
        if test_dest_x < 0 || test_dest_x > 7 || test_dest_y < 0 || test_dest_y > 7 { 
            return (false, true); 
        }

        let (dest_x, dest_y) = (test_dest_x as u8, test_dest_y as u8);
        let existing_dest_sq = self.data._get_by_xy(dest_x, dest_y);
        let (moveable, terminate) = match existing_dest_sq {
            Square::Occupied(dest_piece, dest_square_player) => {(
                can_capture && dest_square_player != self.src_player && (self.can_capture_king || dest_piece != Piece::King), 
                true
            )},
            Square::Blank => {
                (true, false)
            }
        };

        debug!("{},{} moveable={} terminate={}", dest_x, dest_y, moveable, terminate);

        if moveable {
            result.write(&self.make_move_snapshot(dest_x, dest_y, existing_dest_sq, replacement_piece));
        }

        return (moveable, terminate);
    }

    fn make_move_snapshot(
        &self,
        dest_x: u8,
        dest_y: u8,
        existing_dest_square: Square,
        replacement_piece: Option<Piece>
    ) -> MoveSnapshot {
        let mut m: MoveSnapshot = [None; 5];
        m[0] = Some(((self.src_x as u8, self.src_y as u8), BeforeAfterSquares {
            before: Square::Occupied(self.src_piece, self.src_player),
            after: Square::Blank
        }, 0.));
        m[1] = Some(((dest_x, dest_y), BeforeAfterSquares {
            before: existing_dest_square,
            after: Square::Occupied(replacement_piece.unwrap_or(self.src_piece), self.src_player)
        }, 0.));
        return m;
    }

    fn push_promotions(&self, test_dest_x: i8, test_dest_y: i8, can_capture: bool, result: &mut MoveList) -> (bool, bool) {
        let r = self.push(test_dest_x, test_dest_y, can_capture, Some(Piece::Rook), result);
        if r.0 {
            let existing_dest_sq = self.data._get_by_xy(test_dest_x as u8, test_dest_y as u8);
            self.make_move_snapshot(test_dest_x as u8, test_dest_y as u8, existing_dest_sq, Some(Piece::Queen)); 
            self.make_move_snapshot(test_dest_x as u8, test_dest_y as u8, existing_dest_sq, Some(Piece::Bishop)); 
            self.make_move_snapshot(test_dest_x as u8, test_dest_y as u8, existing_dest_sq, Some(Piece::Knight)); 
        }
        return r;
    }

    fn push_pawn(&self, result: &mut MoveList) {
        let (y_delta, jump_row, pre_promote_row) = match self.src_player {
            Player::Black => (1, 1, 6),
            Player::White => (-1, 6, 1)
        };

        let (x, y) = (self.src_x as i8, self.src_y as i8);

        if y == pre_promote_row {
            self.push_promotions(x, y + y_delta, false, result);
        } else {
            if !self.push(x, y + y_delta, false, None, result).1 { // Same as rook ray. If 1-square hop is not blocked, consider 2-square hop.
                if y == jump_row {
                    self.push(x, y + y_delta * 2, false, None, result);
                }
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
                    if y == pre_promote_row {
                        self.push(x + x_delta, y + y_delta, true, None, result);
                    } else {
                        self.push_promotions(x + x_delta, y + y_delta, true, result);
                    }
                }
            }
        }
    }

    fn push_rook(&self, result: &mut MoveList) {
        for _i in 1..=self.src_x {
            let i = self.src_x - _i;
            if self.push(i, self.src_y, true, None, result).1 { break; }
        }
        for i in self.src_x + 1..=7 {
            if self.push(i, self.src_y, true, None, result).1 { break; }
        }
        for _i in 1..=self.src_y {
            let i = self.src_y - _i;
            if self.push(self.src_x, i, true, None, result).1 { break; }
        }
        for i in self.src_y + 1..=7 {
            if self.push(self.src_x, i, true, None, result).1 { break; }
        }
    }

    fn push_bishop(&self, result: &mut MoveList) {

        for i in 1..=self.src_x {
            if self.push(self.src_x - i, self.src_y - i, true, None, result).1 { break; }
        }
        for i in 1..=self.src_x {
            if self.push(self.src_x - i, self.src_y + i, true, None, result).1 { break; }
        }
        for i in 1..=8 - (self.src_x + 1) {
            if self.push(self.src_x + i, self.src_y - i, true, None, result).1 { break; }
        }
        for i in 1..=8 - (self.src_x + 1) {
            if self.push(self.src_x + i, self.src_y + i, true, None, result).1 { break; }
        }
    }

    fn push_knight(&self, result: &mut MoveList) {

        self.push(self.src_x - 1, self.src_y + 2, true, None, result).1;
        self.push(self.src_x - 1, self.src_y - 2, true, None, result).1;

        self.push(self.src_x - 2, self.src_y + 1, true, None, result).1;
        self.push(self.src_x - 2, self.src_y - 1, true, None, result).1;

        self.push(self.src_x + 2, self.src_y + 1, true, None, result).1;
        self.push(self.src_x + 2, self.src_y - 1, true, None, result).1;

        self.push(self.src_x + 1, self.src_y + 2, true, None, result).1;
        self.push(self.src_x + 1, self.src_y - 2, true, None, result).1;
    }

    fn push_queen(&self, result: &mut MoveList) {
        self.push_bishop(result);
        self.push_rook(result);
    }

    fn push_king(&self, result: &mut MoveList) {
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 { continue; }
                self.push(self.src_x + i, self.src_y + j, true, None, result);
            }
        }
    }
}

#[derive(Clone)]
pub struct PlayerState {
    // TODO First thing to switch into bitboards
    pub piece_locs: HashSet<Coord>,
    pub can_castle_oo: bool,
    pub can_castle_ooo: bool
}

impl PlayerState {
    fn new() -> PlayerState {
        PlayerState {
            piece_locs: HashSet::new(),
            can_castle_oo: true,
            can_castle_ooo: true
        }
    }
}

/// Size 2 arrays are indexed by `Player` enum numbers
struct CastleUtils {
    oo_move_snapshots: [MoveSnapshot; 2],
    ooo_move_snapshots: [MoveSnapshot; 2],
    oo_king_traversal_sqs: [[Coord; 3]; 2],
    ooo_king_traversal_sqs: [[Coord; 4]; 2]
}

impl CastleUtils {

    fn get_oo_move_snapshot_for_row(player: Player) -> MoveSnapshot {
        let row = player.get_first_row();
        return [
            Some(((7, row), BeforeAfterSquares { before: Square::Occupied(Piece::Rook, player), after: Square::Blank }, 0.0 )),
            Some(((6, row), BeforeAfterSquares { before: Square::Blank, after: Square::Occupied(Piece::King, player) }, 0.0 )),
            Some(((5, row), BeforeAfterSquares { before: Square::Blank, after: Square::Occupied(Piece::Rook, player) }, 0.0 )),
            Some(((4, row), BeforeAfterSquares { before: Square::Occupied(Piece::King, player), after: Square::Blank }, 0.0 )),
            None
        ];
    }

    fn get_ooo_move_snapshot_for_row(player: Player) -> MoveSnapshot {
        let row = player.get_first_row();
        return [
            Some(((0, row), BeforeAfterSquares { before: Square::Occupied(Piece::Rook, player), after: Square::Blank }, 0.0 )),
            Some(((1, row), BeforeAfterSquares { before: Square::Blank, after: Square::Blank }, 0.0 )),
            Some(((2, row), BeforeAfterSquares { before: Square::Blank, after: Square::Occupied(Piece::King, player) }, 0.0 )),
            Some(((3, row), BeforeAfterSquares { before: Square::Blank, after: Square::Occupied(Piece::Rook, player) }, 0.0 )),
            Some(((4, row), BeforeAfterSquares { before: Square::Occupied(Piece::King, player), after: Square::Blank }, 0.0 ))
        ];
    }

    fn new() -> CastleUtils {
        let white_first_row = Player::get_first_row(Player::White);
        let black_first_row = Player::get_first_row(Player::Black);

        return CastleUtils {
            oo_move_snapshots: [CastleUtils::get_ooo_move_snapshot_for_row(Player::White), CastleUtils::get_ooo_move_snapshot_for_row(Player::White)],
            ooo_move_snapshots: [CastleUtils::get_ooo_move_snapshot_for_row(Player::Black), CastleUtils::get_ooo_move_snapshot_for_row(Player::Black)],
            oo_king_traversal_sqs: [
                [(1, white_first_row), (2, white_first_row), (3, white_first_row)],
                [(1, black_first_row), (2, black_first_row), (3, black_first_row)]
            ],
            ooo_king_traversal_sqs: [
                [(7, white_first_row), (6, white_first_row), (5, white_first_row), (4, white_first_row)],
                [(7, black_first_row), (6, black_first_row), (5, black_first_row), (4, black_first_row)]
            ]
        };
    }
}

#[derive(Clone)]
pub struct Board {
    player_with_turn: Player,
    d: [Square; 64],
    player_state: [PlayerState; 2],
    castle_utils: Rc<CastleUtils>
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
            player_state: [PlayerState::new(), PlayerState::new()],
            castle_utils: Rc::new(CastleUtils::new())
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

    pub fn revert_move(&mut self, m: &MoveSnapshot) {
        for sq in m.iter() {
            if let Some(((x, y), before_after, _)) = sq {
                self.set_by_xy(*x, *y, before_after.before);
            }
        }
    }

    pub fn apply_move(&mut self, m: &MoveSnapshot) {
        for sq in m.iter() {
            if let Some(((x, y), before_after, _)) = sq {
                self.set_by_xy(*x, *y, before_after.after);
            }
        }
    }

    pub fn undo_move(&mut self, moves: &MoveList, index: usize) {
        let m = moves.v[index];
        for square_holder in m.iter() {
            if let Some(((x, y), before_after, _)) = square_holder {
                self.set_by_xy(*x, *y, before_after.before);
            }
        }
    }

    pub fn make_move(&mut self, moves: &MoveList, index: usize) {
        let m = moves.v[index];
        for square_holder in m.iter() {
            if let Some(((x, y), before_after, _)) = square_holder {
                self.set_by_xy(*x, *y, before_after.after);
            }
        }

        self.player_with_turn = self.player_with_turn.get_other_player();
    }

    /// Gets the final set of legal moves
    pub fn get_moves(&mut self, temp_moves: &mut MoveList, result: &mut MoveList) {
        temp_moves.write_index = 0;
        BasicMoveTest::fill_player(self.player_with_turn, self, false, temp_moves);
        BasicMoveTest::filter_check_threats(
            self,
            self.player_with_turn.get_other_player(), 
            temp_moves,
            0,
            temp_moves.write_index,
            result
        );

        // Rewriting temp moves from 0 now
        let player_state = self.get_player_state(self.player_with_turn);
        if player_state.can_castle_oo {
            self.push_castle(
                &self.castle_utils.oo_king_traversal_sqs[self.player_with_turn as usize],
                &self.castle_utils.oo_move_snapshots[self.player_with_turn as usize],
                self.player_with_turn,
                temp_moves,
                result
            );
        }
        if player_state.can_castle_ooo {
            self.push_castle(
                &self.castle_utils.ooo_king_traversal_sqs[self.player_with_turn as usize],
                &self.castle_utils.ooo_move_snapshots[self.player_with_turn as usize],
                self.player_with_turn,
                temp_moves,
                result
            );
        }
    }

    /// Assumes castle has not already been done
    /// Separate from the normal candidate move + check threat pattern
    fn push_castle(
        &mut self,
        king_travel_squares: &[Coord],
        move_snapshot: &MoveSnapshot,
        player_with_turn: Player,
        temp_moves: &mut MoveList,
        result: &mut MoveList
    ) {
        for (x, y) in king_travel_squares.iter() {
            if let Square::Occupied(_, _) = self._get_by_xy(*x, *y) {
                return;
            } 
        }

        for (x, y) in king_travel_squares.iter() {
            self.set_by_xy(*x, *y, Square::Occupied(Piece::King, player_with_turn));
        }
        temp_moves.write_index = 0;
        BasicMoveTest::fill_player(
            player_with_turn.get_other_player(), self, true, temp_moves
        );
        let can_castle = !BasicMoveTest::has_king_capture_move(temp_moves, 0, temp_moves.write_index, player_with_turn)
        for (x, y) in king_travel_squares.iter() {
            self.set_by_xy(*x, *y, Square::Blank);
        }

        if can_castle {
            result.write(move_snapshot);
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
