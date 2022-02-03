use std::fmt::{Error as FmtError, Display, Formatter};

pub struct Bitboard(u64);

impl Bitboard {
    #[inline]
    pub fn set(&mut self, x: u8, y: u8) {
        self.0 |= 1 << (63 - (y * 8 + x));
    }

    #[inline]
    pub fn unset(&mut self, x: u8, y: u8) {
        self.0 &= !(1 << (63 - (y * 8 + x)));
    }

    #[inline]
    pub fn is_set(&self, x: u8, y: u8) -> bool {
        self.0 & (1 << (63 - (y * 8 + x))) != 0
    }

    #[inline]
    pub fn lsb(&self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some(self._lsb())
        }
    }

    #[inline]
    pub fn _lsb(&self) -> u8 {
        // TODO Proper way
        63 - (((self.0 & (!self.0 + 1)) as f64).log2() as u8)
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        for y in 0..8 {
            for x in 0..8 {
                if self.is_set(x, y) {
                    f.write_str("1 ")?;
                } else {
                    f.write_str(". ")?;
                }
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test() {
        let mut bitboard = Bitboard(0);
        bitboard.set(3, 4);
        bitboard.set(5, 6);
        bitboard.set(7, 7);
        bitboard.unset(3, 4);
        bitboard.unset(0, 0);
        bitboard.unset(0, 0);

        assert_eq!(bitboard.is_set(0, 0), false);
        assert_eq!(bitboard.is_set(3, 4), false);
        assert_eq!(bitboard.is_set(5, 6), true);
        assert_eq!(bitboard.is_set(7, 7), true);
        assert_eq!(bitboard.lsb(), Some(63));

        bitboard.unset(7, 7);
        assert_eq!(bitboard.lsb(), Some(53));
        bitboard.unset(5, 6);
        assert_eq!(bitboard.lsb(), None);
        bitboard.set(0, 0);
        assert_eq!(bitboard.lsb(), Some(0));
        bitboard.set(1, 1);
        assert_eq!(bitboard.lsb(), Some(9));

        println!("{} {}", bitboard, bitboard._lsb());
    }
}
