use std::cmp::Ordering;
use std::fmt::{Error as FmtError, Display, Formatter};
use std::ops::Deref;
use super::coords::*;
use super::entities::*;
use crate::{console_log};

/// (bool, bool, u8) = (first to prevent oo, first to prevent ooo, dest sq index)
#[derive(Copy, Clone)]
pub enum MoveDescription {
    Capture(bool, bool, u8),
    Move(bool, bool, u8),
    Oo,
    Ooo,
    Special
}

#[derive(Default, Copy, Clone)]
pub struct BeforeAfterSquares(pub Square, pub Square);

pub type MoveSnapshotSquare = (Coord, BeforeAfterSquares);

// Fairly small bounded size is useable for the most complex move which is castling
pub type MoveSnapshotSquares = [Option<MoveSnapshotSquare>; 5];

// TODO Upgrade callers to inline methods
#[derive(Clone)]
pub struct MoveSnapshot(pub MoveSnapshotSquares, pub f32, pub MoveDescription);

impl MoveSnapshot {

    #[inline]
    pub fn get_squares(&self) -> &MoveSnapshotSquares { &self.0 }
    #[inline]
    pub fn get_eval(&self) -> f32 { self.1 }
    #[inline]
    pub fn get_description(&self) -> &MoveDescription { &self.2 }

    pub fn get_dest_sq(&self) -> Option<&MoveSnapshotSquare> {
        if let MoveDescription::Capture(_, _, dest_sq_index) | MoveDescription::Move(_, _, dest_sq_index) = self.get_description() {
            if let Some(ref x) = self.get_squares()[*dest_sq_index as usize] {
                Some(x)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_src_sq(&self) -> Option<&MoveSnapshotSquare> {
        if let MoveDescription::Capture(_, _, dest_sq_index) | MoveDescription::Move(_, _, dest_sq_index) = self.get_description() {
            for i in 0..self.get_squares().len() {
                if i != *dest_sq_index as usize {
                    if let Some(ref x) = self.get_squares()[i] {
                        return Some(x);
                    }
                }
            }
            None
        } else {
            None
        }
    }
}

impl Deref for MoveSnapshot {
    type Target = MoveSnapshotSquares;

    #[inline]
    fn deref(&self) -> &MoveSnapshotSquares {
        return &self.0;
    }
}

impl Default for MoveSnapshot {
    fn default() -> MoveSnapshot {
        MoveSnapshot([None; 5], 0., MoveDescription::Special)
    }
}

impl Display for MoveSnapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self.get_description() {
            MoveDescription::Capture(p_oo, p_ooo, dest_sq_index) | MoveDescription::Move(p_oo, p_ooo, dest_sq_index) => {
                if let Some((arrival_coord, BeforeAfterSquares(_, after))) = self.get_squares()[*dest_sq_index as usize] {
                    write!(f, "{}{} ({})", after, arrival_coord, self.get_eval())?;
                    if *p_oo {
                        write!(f, " [p_oo]")?;
                    }
                    if *p_ooo {
                        write!(f, " [p_ooo]")?;
                    }
                    return Ok(());
                }

                write!(f, "Error... ({})", self.get_eval())
            },
            MoveDescription::Oo => {
                write!(f, "oo ({})", self.get_eval())
            },
            MoveDescription::Ooo => {
                write!(f, "ooo ({})", self.get_eval())
            },
            _ => {
                write!(f, "Special move?... ({})", self.get_eval())
            }
        }
    }
}

pub struct MoveList {
    v: Vec<MoveSnapshot>,
    pub write_index: usize
}

/// Writers are expected to assume `write_index` is set already to the correct location
impl MoveList {

    pub fn new(capacity: usize) -> Self {
        Self {
            v: Vec::with_capacity(capacity),
            write_index: 0
        }
    }

    #[inline]
    pub fn get_mutable_snapshot(&mut self, i: usize) -> &mut MoveSnapshot {
        &mut self.v[i]
    }

    #[inline]
    pub fn get_v(&self) -> &Vec<MoveSnapshot> {
        &self.v
    }

    #[inline]
    pub fn clone_and_write(&mut self, board_subset: &MoveSnapshot) {
        self.write(board_subset.clone());
    }

    pub fn write(&mut self, board_subset: MoveSnapshot) {
        self.grow_with_access(self.write_index);
        self.v[self.write_index] = board_subset;
        self.write_index += 1;
    }

    fn grow_with_access(&mut self, requested_index: usize) {
        if requested_index >= self.v.len() {
            for _ in 0..requested_index - self.v.len() + 1 {
                self.v.push(MoveSnapshot::default());
            }
        }
    }

    pub fn sort_subset_by_eval(&mut self, start: usize, end_exclusive: usize) {
        let s = &mut self.v[start..end_exclusive];
        s.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
    }

    pub fn write_evals(&mut self, start: usize, end_exclusive: usize, mut to_eval: impl FnMut(&MoveSnapshot) -> f32) {
        for i in start..end_exclusive {
            let m = &mut self.v[i];
            m.1 = to_eval(m);
        }
    }

    pub fn print(&self, start: usize, _end_exclusive: usize) {
        let end_exclusive = if _end_exclusive < self.v.len() {
            _end_exclusive
        } else {
            self.v.len()
        };

        console_log!("[Moves, {}-{}]", start, end_exclusive);
        for i in start..end_exclusive {
            console_log!("{}", self.v[i]);
        }
        console_log!("");
    }
}

