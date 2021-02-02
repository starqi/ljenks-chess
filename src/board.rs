// TODO King, en passante, promotion, castle, castle block
// TODO Rust review - closure types, references to closure types, lifetimes, '_, for loop iter, into_iter, slices, Ref being auto cast
// TODO Split modules, currently too much access between classes
// TODO Revision numbers & board instances, maintaining piece list or not
// TODO File, rank conversion spam

use log::{debug, info, warn, error};
use std::iter::Iterator;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};

pub type Coord = (u8, u8);
pub type CoordList = Vec<Coord>;

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
    XyOutOfBounds(i32, i32),
    MoveListExpired,
    MoveListOutOfBounds(usize, usize),
    RevertMoveExpired(u32, u32)
}

#[derive(Debug)]
pub struct MoveList {
    v: CoordList,
    source: Option<Coord>,
    revision: u32
}

// TODO Same list used on multiple boards?
impl MoveList {
    pub fn new() -> MoveList {
        MoveList { 
            v: Vec::new(),
            source: None,
            revision: 0 
        }
    }

    pub fn is_expired(&self, board: &Board) -> bool {
        self.source.is_none() || self.revision != board.revision
    }

    pub fn get_moves(&self) -> &CoordList {
        &self.v
    }

    pub fn write_move_at_index(&self, board: &Board, index: usize, dest_output: &mut Coord, src_output: &mut Coord) -> Result<(), Error> {

        let src = match self.source {
            None => { return Err(Error::MoveListExpired); },
            Some(ref x) => x 
        };

        if self.revision != board.revision { return Err(Error::MoveListExpired); }

        let dest = match self.v.get(index) {
            None => { return Err(Error::MoveListOutOfBounds(index, self.v.len())); },
            Some(x) => x
        };

        src_output.0 = src.0;
        src_output.1 = src.1;

        dest_output.0 = dest.0;
        dest_output.1 = dest.1;

        Ok(())
    }
}

pub struct CheckThreatTempBuffers {
    move_list: CoordList,
    board: Board
}

impl CheckThreatTempBuffers {
    pub fn new() -> CheckThreatTempBuffers {
        CheckThreatTempBuffers {
            move_list: Vec::new(),
            board: Board::new()
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
    old_player: Player,
    old_revision: u32
}

struct MoveTest<'a, 'b> {
    src_x: u8,
    src_y: u8,
    src_square_player: Player,
    check_threats_and_temp_buffers: Option<(&'b HashSet<Coord>, &'a mut CheckThreatTempBuffers)>,
    data: &'a Board,
    can_capture_king: bool,
    old_move_squares: Option<OldMoveSquares>
}

impl <'a, 'b> MoveTest<'a, 'b> {

    fn new(
        src_x: u8, src_y: u8,
        src_square_player: Player,
        check_threats_and_temp_buffers: Option<(&'b HashSet<Coord>, &'a mut CheckThreatTempBuffers)>,
        data: &'a Board,
        can_capture_king: bool
    ) -> MoveTest<'a, 'b> {
        let mut r = MoveTest {
            src_x, src_y, src_square_player, check_threats_and_temp_buffers, data, can_capture_king, old_move_squares: None
        };
        // TODO Can import for each turn, not each move candidate
        if let Some(t) = &mut r.check_threats_and_temp_buffers {
            t.1.board.import_from(data);
        }
        r
    }

    fn push(&mut self, test_dest_x: i8, test_dest_y: i8, result: &mut CoordList) -> bool {
        self._push(test_dest_x, test_dest_y, true, result)
    }

    fn _push(&mut self, test_dest_x: i8, test_dest_y: i8, capture: bool, result: &mut CoordList) -> bool {
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
                    t.0.iter(),
                    &mut t.1.move_list,
                    &mut |t_x, t_y| Some((t_x, t_y))
                );
                info!("threat={:?}", first_check_threat);

                if first_check_threat.is_none() {
                    result.push((dest_x, dest_y));
                }
            } else {
                result.push((dest_x, dest_y));
            }
        }

        return terminate;
    }

    fn push_rook(&mut self, src_x: i8, src_y: i8, result: &mut CoordList) {
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

    fn push_bishop(&mut self, src_x: i8, src_y: i8, result: &mut CoordList) {
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
    pub revision: u32,
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
            revision: 0,
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
        self.revision = other.revision;

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
        self.revision += 1;
        Ok(())
    }

    pub fn revert_move(&mut self, r: &RevertableMove) -> Result<(), Error> {
        if self.revision != r.old_revision + 1 {
            return Err(Error::RevertMoveExpired(self.revision, r.old_revision));
        }
        self._revert_move(&r.old_move_squares);
        // FIXME Design?
        self.revision = r.old_revision;
        self.player_with_turn = r.old_player;
        Ok(())
    }

    pub fn make_move(
        &mut self,
        moves: &mut MoveList,
        index: usize
    ) -> Result<(), Error> {

        let mut target: Coord = (0, 0);
        let mut source: Coord = (0, 0);
        moves.write_move_at_index(self, index, &mut target, &mut source)?;

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
            self._set_by_xy(target.0, target.1, Square::Occupied(piece, player));
            self._set_by_xy(source.0, source.1, Square::Blank);
        } else {
            panic!("Unexpected blank square in check threats");
        }

        self.revision += 1;
        self.player_with_turn = self.player_with_turn.get_other_player();
        Ok(())
    }

    pub fn get_revertable_move(
        &self,
        moves: &MoveList,
        index: usize
    ) -> Result<RevertableMove, Error> {

        let mut target: Coord = (0, 0);
        let mut source: Coord = (0, 0);
        moves.write_move_at_index(self, index, &mut target, &mut source)?;

        let old_square_a = self.get_by_xy(target.0, target.1)?;
        let old_square_b = self.get_by_xy(source.0, source.1)?;

        Ok(RevertableMove {
            old_move_squares: OldMoveSquares {
                old_square_a: ((target.0, target.1), old_square_a),
                old_square_b: ((source.0, source.1), old_square_b)
            },
            old_revision: self.revision,
            old_player: self.player_with_turn
        })
    }

    pub fn get_moves(
        &self,
        file: char, rank: u8,
        temp_buffers: &mut CheckThreatTempBuffers,
        result: &mut MoveList
    ) -> Result<(), Error> {

        result.v.clear();
        result.revision = self.revision;
        result.source = None;

        let (x, y) = file_rank_to_xy_safe(file, rank)?;
        result.source = Some((x, y));

        let src_square_player = match self.get_by_xy(x, y)? {
            Square::Blank => { return Ok(()); }
            Square::Occupied(_, player) => {
                if player != self.get_player_with_turn() {
                    return Ok(());
                } else {
                    player
                }
            }
        };

        let other_pieces = &self.get_player_state(src_square_player.get_other_player()).piece_locs;
        self._get_moves(
            x, y,
            Some((other_pieces, temp_buffers)),
            false,
            self.player_with_turn,
            &mut result.v
        );

        Ok(())
    }

    fn _revert_move(&mut self, r: &OldMoveSquares) {

        let ((old_x1, old_y1), old_sqr1) = r.old_square_a;
        let ((old_x2, old_y2), old_sqr2) = r.old_square_b;

        self._set_by_xy(old_x1, old_y1, old_sqr1);
        self._set_by_xy(old_x2, old_y2, old_sqr2);
    }

    fn for_each_check_threat<'a, F, R>(
        &self,
        checking_player: Player,
        candidate_squares: impl Iterator<Item = &'a Coord>,
        temp_move_list: &mut CoordList,
        f: &mut F
    ) -> Option<R> where F : FnMut(u8, u8) -> Option<R> {
        for (src_x, src_y) in candidate_squares {
            self._get_moves(*src_x, *src_y, None, true, checking_player, temp_move_list);
            for (dest_x, dest_y) in temp_move_list.iter() {
                if let Square::Occupied(piece, _) = self._get_by_xy(*dest_x, *dest_y) {
                    if piece == Piece::King {
                        if let Some(r) = f(*src_x, *src_y) {
                            return Some(r);
                        }
                    }
                }
            }
        }
        None
    }

    /// A more lax internal definition of a move. 
    /// Context: In the case of checks, to get real moves, need to emulate moves where the king can be captured,
    /// and not restricted to the current player.
    fn _get_moves(
        &self,
        x_u8: u8, y_u8: u8,
        check_threats: Option<(&HashSet<Coord>, &mut CheckThreatTempBuffers)>,
        can_capture_king: bool,
        player_with_turn: Player,
        result: &mut CoordList
    ) {
        result.clear();

        let (piece, square_owner) = match self._get_by_xy(x_u8, y_u8) {
            Square::Blank => { return; },
            Square::Occupied(piece, player) => (piece, player)
        };
        if square_owner != player_with_turn { return; }
        let (x, y) = (x_u8 as i8, y_u8 as i8);

        let mut move_helper = MoveTest::new(x_u8, y_u8, player_with_turn, check_threats, &self, can_capture_king);

        info!("_get_moves src={},{} piece={}", x_u8, y_u8, piece);

        match piece {
            Piece::Pawn => {
                let (y_delta, jump_row) = match square_owner {
                    Player::Black => (1, 1),
                    Player::White => (-1, 6)
                };

                move_helper._push(x, y + y_delta, false, result);
                if y == jump_row {
                    move_helper._push(x, y + y_delta * 2, false, result);
                }

                for x_delta in -1..=1 {
                    if x_delta == 0 { continue; }

                    let x_p_delta: i8 = x + x_delta;
                    let y_p_delta: i8 = y + y_delta;

                    if x_p_delta < 0 || x_p_delta > 7 { continue; }
                    if y_p_delta < 0 || y_p_delta > 7 { continue; }

                    if let Square::Occupied(_, angled_player) = self._get_by_xy(x_p_delta as u8, y_p_delta as u8) {
                        if angled_player != square_owner {
                            move_helper.push(x + x_delta, y + y_delta, result);
                        }
                    }
                }
            },
            Piece::Rook => {
                move_helper.push_rook(x, y, result);
            },
            Piece::Knight => {

                move_helper.push(x - 1, y + 2, result);
                move_helper.push(x - 1, y - 2, result);

                move_helper.push(x - 2, y + 1, result);
                move_helper.push(x - 2, y - 1, result);

                move_helper.push(x + 2, y + 1, result);
                move_helper.push(x + 2, y - 1, result);

                move_helper.push(x + 1, y + 2, result);
                move_helper.push(x + 1, y - 2, result);
            },
            Piece::Bishop => {
                move_helper.push_bishop(x, y, result);
            },
            Piece::Queen => {
                move_helper.push_rook(x, y, result);
                move_helper.push_bishop(x, y, result);
            },
            Piece::King => {
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i == 0 && j == 0 {
                            continue;
                        }
                        move_helper.push(x + i, y + j, result);
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
