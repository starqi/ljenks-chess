// TODO King, en passante, promotion, castle, castle block
// TODO Rust review - closure types, references to closure types, lifetimes, '_, for loop iter, into_iter, slices, Ref being auto cast

use std::cell::{Ref, Cell, RefCell};
use std::iter::Iterator;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};

fn xy_to_file_rank(x: u8, y: u8) -> (char, u8) {
    (std::char::from_u32(x as u32 + ('a' as u32)).unwrap(), 8 - (y as u8))
}

pub fn xy_to_file_rank_safe(x: u8, y: u8) -> Result<(char, u8), Error> {
    if x < 0 || x > 7 || y < 0 || y > 7 {
        return Err(Error::XyOutOfBounds(x, y));
    }
    Ok(xy_to_file_rank(x, y))
}

fn file_rank_to_xy(file: char, rank: u8) -> (u8, u8) {
    let x = file as u32 - 'a' as u32;
    let y = 8 - rank;
    (x as u8, y)
}

// Checks are for public interface
fn file_rank_to_xy_safe(file: char, rank: u8) -> Result<(u8, u8), Error> {
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
    RankOutOfBounds( u8),
    FileOutOfBounds( char),
    XyOutOfBounds(u8, u8),
    MoveListExpired,
    MoveListOutOfBounds( usize, usize)
}

#[derive(Debug)]
pub struct MoveList {
    v: Vec<(u8, u8)>,
    source: Option<(u8, u8)>,
    revision: u32
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList { 
            v: Vec::new(),
            source: None,
            revision: 0 
        }
    }

    pub fn get_moves(&self) -> &Vec<(u8, u8)> {
        &self.v
    }
}

// Must provide buffers for temporary calcs
// (Threats, move list temporary buffer, board copy temporary buffer)
type CheckThreatPieceLocations<'a> = (&'a HashSet<(u8, u8)>, &'a mut Vec<(u8, u8)>, &'a mut BoardData);

pub type CheckThreatTempBuffers<'a> = (&'a mut Vec<(u8, u8)>, &'a mut BoardData);

pub fn make_check_threat_temp_bufs() -> (Vec<(u8, u8)>, BoardData) {
    (Vec::new(), BoardData::new())
}

struct MoveCandidateHelper<'a> {
    src_x: u8,
    src_y: u8,
    src_square_player: Player,
    check_threats: Option<CheckThreatPieceLocations<'a>>,
    data: &'a BoardData,
    can_capture_king: bool
}

impl MoveCandidateHelper<'_> {
    fn push(&mut self, test_dest_x: i8, test_dest_y: i8, result: &mut Vec<(u8, u8)>) -> bool {
        if test_dest_x < 0 || test_dest_x > 7 || test_dest_y < 0 || test_dest_y > 7 {
            return true;
        }

        let (dest_x, dest_y) = (test_dest_x as u8, test_dest_y as u8);

        let (moveable, terminate) = match self.data._get_by_xy(dest_x, dest_y) {
            Square::Occupied(dest_piece, dest_square_player) => {
                (dest_square_player != self.src_square_player && (self.can_capture_king || dest_piece != Piece::King), true)
            },
            Square::Blank => {
                (true, false)
            }
        };

        if moveable {
            if let Some(t) = &mut self.check_threats {
                // Make copy of board
                t.2.import_from(self.data);

                if let Square::Occupied(piece, player) = t.2._get_by_xy(self.src_x, self.src_y) {
                    t.2._set_by_xy(dest_x, dest_y, Square::Occupied(piece, player));
                    t.2._set_by_xy(self.src_x, self.src_y, Square::Blank);
                } else {
                    panic!("Unexpected blank square in check threats");
                }

                let still_has_check_threats = t.2.for_each_check_threat(
                    self.src_square_player.get_other_player(),
                    t.0.iter(),
                    t.1,
                    &mut |_, _| Some(true)
                );

                if still_has_check_threats.is_none() {
                    result.push((dest_x, dest_y));
                }
            } else {
                result.push((dest_x, dest_y));
            }
        }
        return terminate;
    }

    fn push_rook(&mut self, src_x: i8, src_y: i8, result: &mut Vec<(u8, u8)>) {
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

    fn push_bishop(&mut self, src_x: i8, src_y: i8, result: &mut Vec<(u8, u8)>) {
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

pub struct BoardData {
    d: [Square; 64]
}

impl BoardData {

    fn new() -> BoardData {
        BoardData { d: [Square::Blank; 64] }
    }

    fn _set_by_xy(&mut self, x: u8, y: u8, s: Square) {
        self.d[y as usize * 8 + x as usize] = s;
    }

    fn _get_by_xy(&self, x: u8, y: u8) -> Square {
        return self.d[y as usize * 8 + x as usize];
    }
    

    fn import_from(&mut self, other: &BoardData) {
        &self.d[..].copy_from_slice(&other.d);
    }

    fn for_each_check_threat<'a, F, R>(
        &self,
        checking_player: Player,
        candidate_squares: impl Iterator<Item = &'a (u8, u8)>,
        temp_move_list: &mut Vec<(u8, u8)>,
        f: &mut F
    ) -> Option<R> where F : FnMut(u8, u8) -> Option<R> {
        for (src_x, src_y) in candidate_squares {
            self.get_moves(*src_x, *src_y, checking_player, None, true, temp_move_list);
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

    fn get_moves(
        &self,
        x_u8: u8,
        y_u8: u8,
        player_with_turn: Player,
        check_threats: Option<CheckThreatPieceLocations>,
        can_capture_king: bool,
        result: &mut Vec<(u8, u8)>
    ) {
        result.clear();

        let (piece, square_owner) = match self._get_by_xy(x_u8, y_u8) {
            Square::Blank => {
                return;
            },
            Square::Occupied(piece, player) => (piece, player)
        };
        if square_owner != player_with_turn {
            return;
        }
        let (x, y) = (x_u8 as i8, y_u8 as i8);

        let mut move_helper = MoveCandidateHelper {
            src_x: x_u8,
            src_y: y_u8,
            src_square_player: player_with_turn,
            check_threats,
            data: self,
            can_capture_king
        };

        match piece {
            Piece::Pawn => {
                let (y_delta, jump_row) = match square_owner {
                    Player::Black => (1, 1),
                    Player::White => (-1, 6)
                };

                move_helper.push(x, y + y_delta, result);
                if y == jump_row {
                    move_helper.push(x, y + y_delta * 2, result);
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
}

impl Display for BoardData {
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

pub struct Board {
    player_with_turn: Cell<Player>,
    data: RefCell<BoardData>,
    revision: Cell<u32>,
    black_piece_list: RefCell<HashSet<(u8, u8)>>,
    white_piece_list: RefCell<HashSet<(u8, u8)>>
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.data.borrow());
        Ok(())
    }
}

impl Board {
    pub fn new() -> Board {
        let board = Board {
            data: RefCell::new(BoardData::new()),
            player_with_turn: Cell::new(Player::White),
            revision: Cell::new(0),
            black_piece_list: RefCell::new(HashSet::new()),
            white_piece_list: RefCell::new(HashSet::new())
        };

        board.set_main_row(1, Player::White).unwrap();
        board.set_pawn_row(2, Player::White).unwrap();

        board.set_main_row(8, Player::Black).unwrap();
        board.set_pawn_row(7, Player::Black).unwrap();

        board
    }

    pub fn get_player_with_turn(&self) -> Player {
        self.player_with_turn.get()
    }

    pub fn get_piece_locations(&self, player: Player) -> Ref<'_, HashSet<(u8, u8)>> {
        match player {
            Player::White => self.white_piece_list.borrow(),
            Player::Black => self.black_piece_list.borrow()
        }
    }

    pub fn get(&self, file: char, rank: u8) -> Result<Square, Error> {
        let (x, y) = file_rank_to_xy_safe(file, rank)?;
        self.get_by_xy(x, y)
    }

    pub fn get_by_xy(&self, x: u8, y: u8) -> Result<Square, Error> {
        Ok(self.data.borrow()._get_by_xy(x, y))
    }

    pub fn set(&self, file: char, rank: u8, s: Square) -> Result<(), Error> {
        let (x, y) = file_rank_to_xy_safe(file, rank)?;
        self.set_by_xy(x, y, s)?;
        Ok(())
    }

    pub fn set_by_xy(&self, x: u8, y: u8, s: Square) -> Result<(), Error> {

        if let Ok(Square::Occupied(_, occupied_player)) = self.get_by_xy(x, y) {
            let piece_list_rfc = self._get_piece_locations(occupied_player);
            let mut piece_list = piece_list_rfc.borrow_mut();
            piece_list.remove(&(x, y));
        }

        if let Square::Occupied(_, new_player) = s {
            let piece_list_rfc = self._get_piece_locations(new_player);
            let mut piece_list = piece_list_rfc.borrow_mut();
            piece_list.insert((x, y));
        }

        self.data.borrow_mut()._set_by_xy(x, y, s);
        self.revision.set(self.revision.get() + 1);

        Ok(())
    }

    pub fn make_move(&self, moves: &mut MoveList, index: usize, temp_move_list: &mut Vec<(u8, u8)>) -> Result<(), Error> {
        let (src_x, src_y) = match moves.source {
            None => { return Err(Error::MoveListExpired); },
            Some(x) => x 
        };

        if moves.revision != self.revision.get() { return Err(Error::MoveListExpired); }

        let (target_x, target_y) = match moves.v.get(index) {
            None => { return Err(Error::MoveListOutOfBounds(index, moves.v.len())); },
            Some(x) => x
        };

        if let Ok(Square::Occupied(piece, player)) = self.get_by_xy(src_x, src_y) {
            self.set_by_xy(*target_x, *target_y, Square::Occupied(piece, player)).unwrap();
            self.set_by_xy(src_x, src_y, Square::Blank).unwrap();
        } else {
            panic!("Unexpected blank square in check threats");
        }

        self.revision.set(self.revision.get() + 1);
        moves.source = None;

        self.player_with_turn.replace(self.player_with_turn.get().get_other_player());

        Ok(())
    }

    pub fn get_legal_moves(
        &self,
        file: char, rank: u8,
        temp_buffers: CheckThreatTempBuffers,
        result: &mut MoveList
    ) -> Result<(), Error> {

        result.v.clear();
        result.revision = self.revision.get();
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

        let other_pieces = self.get_piece_locations(src_square_player.get_other_player());

        self.data.borrow().get_moves(
            x, y,
            self.player_with_turn.get(),
            Some((&other_pieces, temp_buffers.0, temp_buffers.1)),
            false,
            &mut result.v
        );
        Ok(())
    }

    fn set_pawn_row(&self, rank: u8, player: Player) -> Result<(), Error> {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank, Square::Occupied(Piece::Queen, player))?;
        }
        Ok(())
    }

    fn set_main_row(&self, rank: u8, player: Player) -> Result<(), Error> {
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

    fn _get_piece_locations(&self, player: Player) -> &RefCell<HashSet<(u8, u8)>> {
        match player {
            Player::White => &self.white_piece_list,
            Player::Black => &self.black_piece_list
        }
    }
}

