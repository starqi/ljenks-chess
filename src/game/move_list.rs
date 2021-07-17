use std::cmp::Ordering;
use std::fmt::{Error as FmtError, Display, Formatter};
use super::coords::*;
use super::entities::*;
use crate::{console_log};

#[derive(Clone)]
pub struct BeforeAfterSquares(pub FastCoord, pub Square, pub Square);

pub type PreventOo = bool;
pub type PreventOoo = bool;

#[derive(Clone)]
pub enum MoveDescription {
    Capture([BeforeAfterSquares; 2]),
    Move([BeforeAfterSquares; 2]),
    CastleRelatedCapture([BeforeAfterSquares; 2], PreventOo, PreventOoo),
    CastleRelatedMove([BeforeAfterSquares; 2], PreventOo, PreventOoo),
    Oo,
    Ooo,
    SkipMove
}

impl Default for MoveDescription {
    fn default() -> MoveDescription {
        MoveDescription::SkipMove
    }
}

impl MoveDescription {

    #[inline]
    fn get_sq(&self, i: usize) -> Option<&BeforeAfterSquares> {
        match self {
            MoveDescription::Capture(s) |
            MoveDescription::Move(s) |
            MoveDescription::CastleRelatedMove(s, _, _) |
            MoveDescription::CastleRelatedCapture(s, _, _) => {
                Some(&s[i])
            },
            _ => None
        }
    }

    pub fn get_dest_sq(&self) -> Option<&BeforeAfterSquares> {
        self.get_sq(0)
    }

    pub fn get_src_sq(&self) -> Option<&BeforeAfterSquares> {
        self.get_sq(1)
    }
}

#[derive(Clone, Default)]
pub struct MoveWithEval(MoveDescription, f32);

impl MoveWithEval {
    #[inline]
    pub fn get_description(&self) -> &MoveDescription { &self.0 }
    #[inline]
    pub fn get_eval(&self) -> f32 { self.1 }
}

impl Display for MoveWithEval {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self.get_description() {

            MoveDescription::Capture(sqs) |
            MoveDescription::Move(sqs) |
            MoveDescription::CastleRelatedCapture(sqs, _, _) | 
            MoveDescription::CastleRelatedMove(sqs, _, _) => {

                let BeforeAfterSquares(fast_coord, _, after_sq) = sqs[1];
                // Since a piece is on the after square, after_sq will stringify to eg. k, K, p, P, then it becomes eg. Ke2
                write!(f, "{}{} ({})", after_sq, fast_coord, self.get_eval())?;
            },

            MoveDescription::Oo => {
                write!(f, "oo ({})", self.get_eval())?;
            },
            MoveDescription::Ooo => {
                write!(f, "ooo ({})", self.get_eval())?;
            },
            MoveDescription::SkipMove => {
                write!(f, "skip ({})", self.get_eval())?;
            }
        }

        match self.get_description() {
            MoveDescription::CastleRelatedCapture(sqs, p_oo, p_ooo) | 
            MoveDescription::CastleRelatedMove(sqs, p_oo, p_ooo) => {
                if *p_oo { write!(f, " [p_oo]")?; }
                if *p_ooo { write!(f, " [p_ooo]")?; }
            },
            _ => {}
        }

        Ok(())
    }
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
    pub fn get_v(&self) -> &Vec<MoveWithEval> {
        &self.v
    }

    #[inline]
    pub fn clone_and_write(&mut self, m: &MoveWithEval) {
        self.write(m.clone());
    }

    pub fn write(&mut self, board_subset: MoveWithEval) {
        self.grow_with_access(self.write_index);
        self.v[self.write_index] = board_subset;
        self.write_index += 1;
    }

    fn grow_with_access(&mut self, requested_index: usize) {
        if requested_index >= self.v.len() {
            for _ in 0..requested_index - self.v.len() + 1 {
                self.v.push(MoveWithEval::default());
            }
        }
    }

    pub fn sort_subset_by_eval(&mut self, start: usize, end_exclusive: usize) {
        let s = &mut self.v[start..end_exclusive];
        s.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
    }

    pub fn write_evals(&mut self, start: usize, end_exclusive: usize, mut to_eval: impl FnMut(&MoveWithEval) -> f32) {
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

