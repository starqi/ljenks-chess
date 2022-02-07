use std::fmt::{Error as FmtError, Display, Formatter};
use super::super::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Bitboard(pub u64);

/// Assumes top left corner index is 0. 63 is bottom right.
impl Bitboard {

    pub fn slow_safe_set(&mut self, x: i8, y: i8) -> bool {
        if x < 0 || y < 0 || x >= 8 || y >= 8 { return false; }
        self.set(x as u8, y as u8);
        true
    }

    #[inline]
    pub fn set(&mut self, x: u8, y: u8) {
        self.set_index(y * 8 + x);
    }

    #[inline]
    pub fn unset(&mut self, x: u8, y: u8) {
        self.unset_index(y * 8 + x);
    }

    #[inline]
    pub fn unset_index(&mut self, index: u8) {
        self.0 &= !(1 << (63 - index));
    }

    #[inline]
    pub fn set_index(&mut self, index: u8) {
        self.0 |= 1 << (63 - index);
    }

    #[inline]
    pub fn is_set(&self, x: u8, y: u8) -> bool {
        self.0 & (1 << (63 - (y * 8 + x))) != 0
    }

    #[inline]
    pub fn lsb_to_index(&self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some(self._lsb_to_index())
        }
    }

    #[inline]
    pub fn msb_to_index(&self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some(self._msb_to_index())
        }
    }

    /// Precondition: Bitboard value is not 0
    #[inline]
    pub fn _lsb_to_index(&self) -> u8 {
        // TODO Write intuition
        BITBOARD_PRESETS.debruijn_indices[(((self.0 & (!self.0 + 1)).wrapping_mul(BITBOARD_PRESETS.debruijn_sequence)) >> 58) as usize]
    }

    /// Precondition: Bitboard value is not 0
    #[inline]
    pub fn _msb_to_index(&self) -> u8 {
        // Set to 1 the lower bits from the MSB, doubling the amount of bits set each time,
        // then use add carry propagation to isolate the MSB, rest is same as LSB.
        let mut x = self.0;
        x |= x >> 1; 
        x |= x >> 2;
        x |= x >> 4;
        x |= x >> 8;
        x |= x >> 16;
        x |= x >> 32;
        x >>= 1;
        x = x.wrapping_add(1);
        BITBOARD_PRESETS.debruijn_indices[((x.wrapping_mul(BITBOARD_PRESETS.debruijn_sequence)) >> 58) as usize]
    }

    pub fn consume_loop_indices(&mut self, mut cb: impl FnMut(u8) -> ()) {
        while self.0 != 0 {
            let index = self._lsb_to_index();
            cb(index);
            self.unset_index(index);
        }
    }

    pub fn consume_loop_indices2(&mut self, mut cb: impl FnMut(u8) -> bool) -> bool {
        while self.0 != 0 {
            let index = self._lsb_to_index();
            if cb(index) { return true; }
            self.unset_index(index);
        }
        false
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
    fn set_and_lsb_test() {
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
        assert_eq!(bitboard.lsb_to_index(), Some(63));

        bitboard.unset(7, 7);
        assert_eq!(bitboard.lsb_to_index(), Some(53));
        bitboard.unset(5, 6);
        assert_eq!(bitboard.lsb_to_index(), None);
        bitboard.set(0, 0);
        assert_eq!(bitboard.lsb_to_index(), Some(0));
        bitboard.set(1, 1);
        assert_eq!(bitboard.lsb_to_index(), Some(9));
    }

    #[test]
    fn msb_test() {
        let mut bitboard = Bitboard(0);
        bitboard.set(5, 6);
        bitboard.set(7, 7);

        assert_eq!(bitboard.msb_to_index(), Some(53));
        bitboard.unset(5, 6);
        assert_eq!(bitboard.msb_to_index(), Some(63));
        bitboard.unset(7, 7);
        assert_eq!(bitboard.msb_to_index(), None);
    }

    #[test]
    fn consume_loop_test() {
        let mut bitboard = Bitboard(0);
        bitboard.set(3, 4);
        bitboard.set(5, 6);
        bitboard.set(7, 7);

        let mut v: Vec<u8> = Vec::new();
        bitboard.consume_loop_indices(|index| {
            v.push(index);
        });

        assert_eq!(v, [63, 53, 35]);
        assert_eq!(bitboard.0, 0);
    }
}
