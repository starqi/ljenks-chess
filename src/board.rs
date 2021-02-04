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
    write_index: usize
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

    pub fn get_write_index(&self) -> usize {
        self.write_index
    }

    pub fn set_write_index(&mut self, w: usize) {
        if w >= self.v.len() { panic!("Out of bounds write index"); }
        self.write_index = w;
    }

    pub fn write(&mut self, from: Coord, to: Coord, eval: i32) {
        if self.write_index >= self.v.len() {
            self.v.push(((0, 0), (0, 0), 0));
        }
        let mut item = self.v[self.v.len() - 1];
        item.0 = from;
        item.1 = to;
        item.2 = eval;
        self.write_index += 1;
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
                write!(f, ".")
            },
            Square::Occupied(piece, player) => {
                piece.custom_fmt(f, *player == Player::Black)
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

pub struct CheckThreatTempBuffers<'a> {
    moves: MoveList,
    board: Board,
    move_test: MoveTest<'a, 'a>
}

impl <'a> CheckThreatTempBuffers<'a> {
    pub fn new() -> CheckThreatTempBuffers<'a> {
        let board = Board::new();
        CheckThreatTempBuffers {
            moves: MoveList::new(50),
            board,
            move_test: MoveTest::new(false, &mut board)
        }
    }
}

#[derive(Default)]
struct OldMoveSquares {
    old_square_a: (Coord, Square),
    old_square_b: (Coord, Square)
}

pub struct RevertableMove {
    old_move_squares: OldMoveSquares,
    old_player: Player
}

struct MoveTest<'a, 'b> where 'a : 'b {
    src_x: u8, src_y: u8,
    src_square_player: Player,
    check_threats_and_temp_buffers: Option<(&'b HashSet<Coord>, &'a mut CheckThreatTempBuffers<'a>)>,
    data: &'a Board,
    can_capture_king: bool,
    old_move_squares: Option<OldMoveSquares>
}

impl <'a, 'b> MoveTest<'a, 'b> {

    fn new(does_checks: bool, data: &'a Board) -> MoveTest<'a, 'b> {
        let mut r = MoveTest {
            src_x: 0,
            src_y: 0, 
            src_square_player: Player::White,
            check_threats_and_temp_buffers: if does_checks {
                Some((H))
            } else {
                None
            },
            data,
            can_capture_king: true,
            old_move_squares: None
        };
        // TODO Can import for each turn, not each move candidate
        if let Some(t) = &mut r.check_threats_and_temp_buffers {
            t.1.board.import_from(data);
        }
        r
    }

    fn push(
        &mut self,
        test_dest_x: i8,
        test_dest_y: i8,
        result: &mut MoveList
    ) -> bool {
        self._push(test_dest_x, test_dest_y, true, result)
    }

    fn _push(&mut self, test_dest_x: i8, test_dest_y: i8, capture: bool, result: &mut MoveList) -> bool {
        if test_dest_x < 0 || test_dest_x > 7 || test_dest_y < 0 || test_dest_y > 7 {
            return true;
        }

        let (dest_x, dest_y) = (test_dest_x as u8, test_dest_y as u8);

        let (moveable, terminate) = match self.data._get_by_xy(dest_x, dest_y) {
            Square::Occupied(dest_piece, dest_square_player) => {
                (
                    capture && dest_square_player != self.src_square_player && (self.can_capture_king || dest_piece != Piece::King), 
                    true
                )
            },
            Square::Blank => {
                (true, false)
            }
        };

        debug!("{},{} moveable={} terminate={}", dest_x, dest_y, moveable, terminate);

        if moveable {
            if let Some(t) = &mut self.check_threats_and_temp_buffers {

                // Revert board to constructor init state
                if let Some(ref oms) = self.old_move_squares {
                    t.1.board._revert_move(oms);
                }

                if let Square::Occupied(piece, player) = t.1.board._get_by_xy(self.src_x, self.src_y) {

                    // Get revert targets buffer
                    let mut oms = self.old_move_squares.get_or_insert(Default::default());

                    oms.old_square_a.0.0 = dest_x;
                    oms.old_square_a.0.1 = dest_y;
                    oms.old_square_a.1 = t.1.board._get_by_xy(dest_x, dest_y);

                    oms.old_square_b.0.0 = self.src_x;
                    oms.old_square_b.0.1 = self.src_y;
                    oms.old_square_b.1 = t.1.board._get_by_xy(self.src_x, self.src_y);

                    t.1.board._set_by_xy(dest_x, dest_y, Square::Occupied(piece, player));
                    t.1.board._set_by_xy(self.src_x, self.src_y, Square::Blank);
                } else {
                    panic!("Unexpected blank square in check threats");
                }

                info!("calc check threats, checker={:?}", self.src_square_player.get_other_player());
                info!("\n{}", t.1.board);

                let first_check_threat = t.1.board.for_each_check_threat(
                    self.src_square_player.get_other_player(),
                    &mut t.1.moves,
                    &mut |t_x, t_y| Some((t_x, t_y))
                );
                info!("threat={:?}", first_check_threat);

                if first_check_threat.is_none() {
                    result.write((self.src_x, self.src_y), (dest_x, dest_y), 0);
                }
            } else {
                result.write((self.src_x, self.src_y), (dest_x, dest_y), 0);
            }
        }

        return terminate;
    }

    fn push_rook(&mut self, src_x: i8, src_y: i8, result: &mut MoveList) {
        for _i in 1..=src_x {
            let i = src_x - _i;
            if self.push(i, src_y, result) { break; }
        }
        for i in src_x + 1..=7 {
            if self.push(i, src_y, result) { break; }
        }
        for _i in 1..=src_y {
            let i = src_y - _i;
            if self.push(src_x, i, result) { break; }
        }
        for i in src_y + 1..=7 {
            if self.push(src_x, i, result) { break; }
        }
    }

    fn push_bishop(&mut self, src_x: i8, src_y: i8, result: &mut MoveList) {
        for i in 1..=src_x {
            if self.push(src_x - i, src_y - i, result) { break; }
        }
        for i in 1..=src_x {
            if self.push(src_x - i, src_y + i, result) { break; }
        }
        for i in 1..=8 - (src_x + 1) {
            if self.push(src_x + i, src_y - i, result) { break; }
        }
        for i in 1..=8 - (src_x + 1) {
            if self.push(src_x + i, src_y + i, result) { break; }
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

    pub fn revert_move(&mut self, r: &RevertableMove) -> Result<(), Error> {
        self._revert_move(&r.old_move_squares);
        self.player_with_turn = r.old_player;
        Ok(())
    }

    pub fn make_move(&mut self, moves: &mut MoveList, index: usize) {

        let m = match moves.get_v().get(index) {
            None => panic!("Make move out of bounds {}/{}", index, moves.get_v().len()),
            Some(_m) => _m
        };

        let source = m.0;
        let dest = m.1;

        if let Ok(Square::Occupied(piece, player)) = self.get_by_xy(source.0, source.1) {

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
            self._set_by_xy(dest.0, dest.1, Square::Occupied(piece, player));
            self._set_by_xy(source.0, source.1, Square::Blank);
        } else {
            panic!("Unexpected blank square in check threats");
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
    pub fn get_moves(
        &self,
        temp_check_buffers: &mut CheckThreatTempBuffers,
        result: &mut MoveList
    ) {
        let other_pieces = &self.get_player_state(self.get_player_with_turn().get_other_player()).piece_locs;
        let temp_move_test = MoveTest::new(&mut Some((other_pieces, temp_check_buffers)), &self);
        self._get_moves(self.get_player_with_turn(), false, &mut temp_move_test, result);
    }

    fn _revert_move(&mut self, r: &OldMoveSquares) {

        let ((old_x1, old_y1), old_sqr1) = r.old_square_a;
        let ((old_x2, old_y2), old_sqr2) = r.old_square_b;

        self._set_by_xy(old_x1, old_y1, old_sqr1);
        self._set_by_xy(old_x2, old_y2, old_sqr2);
    }

    /// Move lists which are temp buffers are auto cleared, because caller should not care what's inside
    fn for_each_check_threat<'a, F, R>(
        &self,
        checking_player: Player,
        temp_move_test: &mut MoveTest,
        temp_moves: &mut MoveList,
        f: &mut F
    ) -> Option<R> where F : FnMut(u8, u8) -> Option<R> {

        temp_moves.set_write_index(0);
        self._get_moves(checking_player, true, temp_move_test, temp_moves);
        let end_exclusive = temp_moves.get_write_index();

        for i in 0..end_exclusive {
            let ((src_x, src_y), (dest_x, dest_y), _) = temp_moves.get_v()[i];
            if let Square::Occupied(piece, _) = self._get_by_xy(dest_x, dest_y) {
                if piece == Piece::King {
                    if let Some(r) = f(src_x, src_y) {
                        return Some(r);
                    }
                }
            }
        }
        None
    }

    /// Only constructor arguments of `temp_move_test` are kept
    fn _get_moves(
        &self,
        player_with_turn: Player,
        can_capture_king: bool,
        temp_move_test: &mut MoveTest,
        result: &mut MoveList
    ) {
        temp_move_test.src_square_player = player_with_turn;
        temp_move_test.can_capture_king = can_capture_king;

        for (x_u8, y_u8) in &self.get_player_state(player_with_turn).piece_locs {

            temp_move_test.src_x = *x_u8;
            temp_move_test.src_y = *y_u8;

            let (piece, square_owner) = match self._get_by_xy(*x_u8, *y_u8) {
                Square::Blank => { return; },
                Square::Occupied(piece, player) => (piece, player)
            };
            if square_owner != player_with_turn { return; }
            let (x, y) = (*x_u8 as i8, *y_u8 as i8);

            info!("_get_moves src={},{} piece={}", x_u8, y_u8, piece);

            match piece {
                Piece::Pawn => {
                    let (y_delta, jump_row) = match square_owner {
                        Player::Black => (1, 1),
                        Player::White => (-1, 6)
                    };

                    temp_move_test._push(x, y + y_delta, false, result);
                    if y == jump_row {
                        temp_move_test._push(x, y + y_delta * 2, false, result);
                    }

                    for x_delta in -1..=1 {
                        if x_delta == 0 { continue; }

                        let x_p_delta: i8 = x + x_delta;
                        let y_p_delta: i8 = y + y_delta;

                        if x_p_delta < 0 || x_p_delta > 7 { continue; }
                        if y_p_delta < 0 || y_p_delta > 7 { continue; }

                        if let Square::Occupied(_, angled_player) = self._get_by_xy(x_p_delta as u8, y_p_delta as u8) {
                            if angled_player != square_owner {
                                temp_move_test.push(x + x_delta, y + y_delta, result);
                            }
                        }
                    }
                },
                Piece::Rook => {
                    temp_move_test.push_rook(x, y, result);
                },
                Piece::Knight => {

                    temp_move_test.push(x - 1, y + 2, result);
                    temp_move_test.push(x - 1, y - 2, result);

                    temp_move_test.push(x - 2, y + 1, result);
                    temp_move_test.push(x - 2, y - 1, result);

                    temp_move_test.push(x + 2, y + 1, result);
                    temp_move_test.push(x + 2, y - 1, result);

                    temp_move_test.push(x + 1, y + 2, result);
                    temp_move_test.push(x + 1, y - 2, result);
                },
                Piece::Bishop => {
                    temp_move_test.push_bishop(x, y, result);
                },
                Piece::Queen => {
                    temp_move_test.push_rook(x, y, result);
                    temp_move_test.push_bishop(x, y, result);
                },
                Piece::King => {
                    for i in -1..=1 {
                        for j in -1..=1 {
                            if i == 0 && j == 0 {
                                continue;
                            }
                            temp_move_test.push(x + i, y + j, result);
                        }
                    }
                }
            }
        }
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
        self.set_main_row(1, Player::White).unwrap();
        self.set_uniform_row(2, Player::White, Piece::Pawn).unwrap();
        self.set_main_row(8, Player::Black).unwrap();
        self.set_uniform_row(7, Player::Black, Piece::Pawn).unwrap();
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
