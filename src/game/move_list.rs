use std::cmp::Ordering;
use std::fmt::{Error as FmtError, Display, Formatter};
use std::ops::Deref;
use super::coords::*;
use super::entities::*;

#[derive(Default, Copy, Clone)]
pub struct BeforeAfterSquares(pub Square, pub Square);

pub type Eval = f32;
pub type MoveSnapshotSquare = (Coord, BeforeAfterSquares);

// Fairly small bounded size is useable for the most complex move which is castling
pub type MoveSnapshotSquares = [Option<MoveSnapshotSquare>; 5];

#[derive(Copy, Clone)]
pub struct MoveSnapshot(pub MoveSnapshotSquares, pub Eval);

impl PartialEq for MoveSnapshot {
    fn eq(&self, other: &MoveSnapshot) -> bool {
        self.1 == other.1
    }
}

impl PartialOrd for MoveSnapshot {
    fn partial_cmp(&self, other: &MoveSnapshot) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Eq for MoveSnapshot {}

impl Ord for MoveSnapshot {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.1 == other.1 {
            // Allowing float equality comparison makes no difference here
            Ordering::Equal
        } else {
            self.1.partial_cmp(&other.1).unwrap()
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
        MoveSnapshot([None; 5], 0.)
    }
}

impl Display for MoveSnapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let mut square_count = 0;
        for sq in &self.0 {
            if sq.is_some() {
                square_count += 1;
            }
        }

        if square_count == 2 {

            let mut arrival: Option<&MoveSnapshotSquare> = None;
            let mut departed: Option<&MoveSnapshotSquare> = None;
            let mut departed_i: u8 = 0;

            let mut i: u8 = 0;
            for sq in &self.0 {
                if let Some(k@(_, BeforeAfterSquares(Square::Occupied(_, _), Square::Blank))) = sq {
                    departed = Some(k);
                    departed_i = i;
                    break;
                }
                i += 1;
            }

            if departed.is_some() {
                i = 0;
                for sq in &self.0 {
                    if i != departed_i {
                        if let Some(sq2) = sq {
                            arrival = Some(sq2);
                            break;
                        }
                    }
                    i += 1;
                }
            }

            if let Some((arrival_coord, BeforeAfterSquares(_, after))) = arrival {
                if let Some((departed_coord, _)) = departed {
                    return write!(f, "{}@ {} to {} ({})", after, departed_coord, arrival_coord, self.1);
                }
            }
        } 

        write!(f, "Special move involving {} squares, {}", square_count, self.1)
    }
}

pub struct MoveList {
    v: Vec<MoveSnapshot>,
    pub write_index: usize
}

/// Writers are expected to assume `write_index` is set already to the correct location
impl MoveList {

    pub fn new(capacity: usize) -> MoveList {
        MoveList {
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
    pub fn copy_and_write(&mut self, board_subset: &MoveSnapshot) {
        self.write(*board_subset);
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

    pub fn sort_subset(&mut self, start: usize, end_exclusive: usize) {
        unsafe {
            let mut v = Vec::from_raw_parts(&mut self.v[start] as *mut MoveSnapshot, end_exclusive - start, end_exclusive - start);
            v.sort_unstable();
        }
    }

    pub fn print(&self, start: usize, _end_exclusive: usize) {
        let end_exclusive = if _end_exclusive < self.v.len() {
            _end_exclusive
        } else {
            self.v.len()
        };

        println!("[Moves, {}-{}]", start, end_exclusive);
        // TODO Minor: Optional rev, for principal variation
        for i in (start..end_exclusive).rev() {
            println!("{}", self.v[i]);
        }
        println!("");
    }
}

