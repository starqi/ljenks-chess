use std::cmp::Ordering;
use super::coords::*;
use super::entities::*;

#[derive(Clone)]
pub struct BeforeSquare(pub FastCoord, pub Square);

#[derive(Clone)]
pub struct BeforeAfterSquare(pub FastCoord, pub Square, pub Square);

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CastleType {
    Oo = 0, Ooo
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MoveMetadata {
    None = 0, DoublePawnJump, EnPassant, Promotion // TODO Promotion
}

// Keep minimal in size, to make move generation fast, and move execution slower
// TOOD (Minor) Copy or clone?
#[derive(Clone)]
pub enum MoveDescription {
    /// (From, to, metadata)
    NormalMove(FastCoord, FastCoord, MoveMetadata),
    Castle(CastleType),
    SkipMove
}

impl Default for MoveDescription {
    fn default() -> MoveDescription {
        MoveDescription::SkipMove
    }
}

/// (MoveDescription, ordering score not eval)
#[derive(Clone, Default)]
pub struct MoveWithEval(pub MoveDescription, pub i32);

impl MoveWithEval {
    #[inline]
    pub fn description(&self) -> &MoveDescription { &self.0 }
    #[inline]
    pub fn ordering_score(&self) -> i32 { self.1 }
}

pub struct MoveList {
    v: Vec<MoveWithEval>,
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
    pub fn get_mutable_snapshot(&mut self, i: usize) -> &mut MoveWithEval {
        &mut self.v[i]
    }

    #[inline]
    pub fn v(&self) -> &Vec<MoveWithEval> {
        &self.v
    }

    #[inline]
    pub fn v_unsafe(&mut self) -> &mut Vec<MoveWithEval> {
        &mut self.v
    }

    #[inline]
    pub fn write_clone(&mut self, m: &MoveWithEval) {
        self.write(m.clone());
    }

    pub fn write(&mut self, m: MoveWithEval) {
        self.grow_with_access(self.write_index);
        self.v[self.write_index] = m;
        self.write_index += 1;
    }

    fn grow_with_access(&mut self, requested_index: usize) {
        if requested_index >= self.v.len() {
            for _ in 0..requested_index - self.v.len() + 1 {
                self.v.push(MoveWithEval::default());
            }
        }
    }
}

