use std::fmt::{Display, Formatter, self};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FastCoord(u8);

impl FastCoord {

    #[inline]
    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn to_coord(&self) -> Coord {
        let y = (self.0 / 8) as u8;
        let x = self.0 % 8;
        Coord(x, y)
    }

    pub fn from_xy(x: u8, y: u8) -> FastCoord {
        FastCoord(y * 8 + x)
    }

    pub fn from_coord(coord: &Coord) -> FastCoord {
        FastCoord(coord.1 * 8 + coord.0)
    }
}

impl Display for FastCoord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        self.to_coord().fmt(f)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    RankOutOfBounds(u8),
    FileOutOfBounds(char),
    XyOutOfBounds(i32, i32)
}

/// x, y
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct Coord(pub u8, pub u8);

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        if let Ok((letter, num)) = xy_to_file_rank_safe(self.0 as i32, self.1 as i32) {
            write!(f, "{}{}", letter, num)
        } else {
            write!(f, "(Invalid coord)")
        }
    }
}

pub fn xy_to_file_rank(x: u8, y: u8) -> (char, u8) {
    (std::char::from_u32(x as u32 + ('a' as u32)).unwrap(), 8 - (y as u8))
}

#[inline]
pub fn check_i32_xy(x: i32, y: i32) -> Result<(), Error> {
    if x < 0 || x > 7 || y < 0 || y > 7 {
        Err(Error::XyOutOfBounds(x, y))
    } else {
        Ok(())
    }
}

pub fn xy_to_file_rank_safe(x: i32, y: i32) -> Result<(char, u8), Error> {
    check_i32_xy(x, y)?;
    Ok(xy_to_file_rank(x as u8, y as u8))
}

pub fn file_rank_to_xy(file: char, rank: u8) -> Coord {
    let x = file as u32 - 'a' as u32;
    let y = 8 - rank;
    Coord(x as u8, y)
}

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

