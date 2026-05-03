use std::fmt::{Error as FmtError, Display, Formatter};

// Unenforced DEPENDENCIES assuming the number -> enum translation: 
// - index.js numToLetter
// - evaluation.rs
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Piece {
    Pawn = 0, Knight, Bishop, Rook, Queen, King
}

impl Piece {
    pub fn from_number(n: u8) -> Self {
        match n % 6 {
            0 => Piece::Pawn,
            1 => Piece::Knight,
            2 => Piece::Bishop,
            3 => Piece::Rook,
            4 => Piece::Queen,
            5 => Piece::King,
            _ => unreachable!(),
        }
    }

    pub fn custom_fmt(&self, f: &mut Formatter<'_>, is_lower: bool) -> Result<(), FmtError> {
        let s = match self {
            Piece::Pawn => "P",
            Piece::Knight => "N",
            Piece::Bishop => "B",
            Piece::Rook => "R",
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        self.custom_fmt(f, true)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player { 
    White = 0, Black
}

static PLAYER_TO_OTHER_PLAYER: [Player; 2] = [Player::Black, Player::White];
static PLAYER_TO_MULTIPLIER: [i32; 2] = [1, -1];
static PLAYER_TO_FIRST_ROW: [u8; 2] = [7, 0];
static PLAYER_TO_LAST_ROW: [u8; 2] = [0, 7];

impl Player {

    #[inline]
    pub fn other_player(self) -> Player {
        PLAYER_TO_OTHER_PLAYER[self as usize]
    }

    #[inline]
    pub fn first_row(self) -> u8 {
        PLAYER_TO_FIRST_ROW[self as usize]
    }

    #[inline]
    pub fn last_row(self) -> u8 {
        PLAYER_TO_LAST_ROW[self as usize]
    }

    #[inline]
    pub fn multiplier(self) -> i32 {
        PLAYER_TO_MULTIPLIER[self as usize]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Square {
    Occupied(Piece, Player), Blank
}

impl Default for Square {
    fn default() -> Self {
        Square::Blank
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            Square::Blank => {
                write!(f, ". ")
            },
            Square::Occupied(piece, player) => {
                let r = piece.custom_fmt(f, *player == Player::Black);
                write!(f, " ")?;
                r
            }
        }
    }
}
