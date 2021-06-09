use std::fmt::{Error as FmtError, Display, Formatter, self};

#[repr(u8)]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        self.custom_fmt(f, true)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player { 
    White = 0, Black
}

static player_to_other_player: [Player; 2] = [Player::Black, Player::White];
static player_to_multiplier: [f32; 2] = [1., -1.];
static player_to_first_row: [u8; 2] = [7, 1];

impl Player {

    #[inline]
    pub fn get_other_player(self) -> Player {
        player_to_other_player[self as usize]
    }

    #[inline]
    pub fn get_first_row(self) -> u8 {
        player_to_first_row[self as usize]
    }

    #[inline]
    pub fn get_multiplier(self) -> f32 {
        player_to_multiplier[self as usize]
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
                write!(f, " ")?;
                r
            }
        }
    }
}
