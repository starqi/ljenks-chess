use super::evaluation::{evaluate_piece, add_mobility_to_vec};
use super::super::game::move_list::{MoveWithEval, MoveList, MoveDescription};
use super::super::game::entities::*;
use super::super::game::board::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BucketType {
    WinningCapture,
    EqualCapture,
    NonCapture,
    LosingCapture,
}

/// Bucket sorts moves for qsearch and normal search.
/// Assumes qsearch non-captures are all checks. 
/// After sorting, rewrites sorted moves back into the move list. 
/// Perhaps only a subset of all the moves are needed (currently qsearch discards losing captures), in which a new end exclusive is returned.
/// Normal search sort still populates `MoveWithEval` score field for LMR, where poor moves have score >= `NORMAL_SEARCH_POOR_MOVE_MIN_SCORE`. 
pub struct MoveBuckets {
    pub winning_captures: Vec<MoveWithEval>,
    pub equal_captures: Vec<MoveWithEval>,
    pub non_captures: Vec<MoveWithEval>,
    pub losing_captures: Vec<MoveWithEval>,
}

pub const NORMAL_SEARCH_POOR_MOVE_MIN_SCORE: i32 = 2;

impl MoveBuckets {
    pub fn new() -> Self {
        Self {
            winning_captures: Vec::new(),
            equal_captures: Vec::new(),
            non_captures: Vec::new(),
            losing_captures: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.winning_captures.clear();
        self.equal_captures.clear();
        self.non_captures.clear();
        self.losing_captures.clear();
    }

    pub fn reorder_and_assign_scores_qsearch(&self, move_list: &mut MoveList, start: usize) -> usize {
        let buckets = [&self.winning_captures, &self.non_captures, &self.equal_captures];

        let mut current_index = start;
        for b in buckets {
            for move_clone in b {
                *move_list.get_mutable_snapshot(current_index) = move_clone.clone();
                current_index += 1;
            }
        }
        current_index
    }

    pub fn reorder_and_assign_scores(&self, move_list: &mut MoveList, start: usize) -> usize {
        let buckets = [&self.winning_captures, &self.equal_captures, &self.non_captures, &self.losing_captures];

        let mut current_index = start;
        for bucket_idx in 0..4 {
            for move_clone in buckets[bucket_idx] {
                let mut modified_move = move_clone.clone();
                // See above docs for `NORMAL_SEARCH_POOR_MOVE_MIN_SCORE`.
                modified_move.1 = bucket_idx as i32;
                *move_list.get_mutable_snapshot(current_index) = modified_move;
                current_index += 1;
            }
        }
        current_index
    }

    pub fn group_moves_qsearch(&mut self, board: &Board, move_list: &MoveList, start: usize, end_exclusive: usize) {
        self.clear();
        for i in start..end_exclusive {
            let m = &move_list.v()[i];
            let bucket = self.determine_bucket(board, m);
            match bucket {
                BucketType::WinningCapture => self.winning_captures.push(m.clone()),
                BucketType::EqualCapture => self.equal_captures.push(m.clone()),
                BucketType::NonCapture => self.non_captures.push(m.clone()),
                BucketType::LosingCapture => {}, // Skip losing captures for qsearch
            }
        }
        // No mobility for qsearch
    }

    pub fn group_moves_normal(&mut self, board: &Board, move_list: &MoveList, start: usize, end_exclusive: usize) {
        self.clear();
        for i in start..end_exclusive {
            let m = &move_list.v()[i];
            let bucket = self.determine_bucket(board, m);
            match bucket {
                BucketType::WinningCapture => self.winning_captures.push(m.clone()),
                BucketType::EqualCapture => self.equal_captures.push(m.clone()),
                BucketType::NonCapture => self.non_captures.push(m.clone()),
                BucketType::LosingCapture => self.losing_captures.push(m.clone()),
            }
        }
        add_mobility_to_vec(board, &mut self.non_captures);
        if self.non_captures.len() > 10 {
            // Descending order
            self.non_captures.select_nth_unstable_by(10, |a, b| b.1.cmp(&a.1));
            self.non_captures[..10].sort_by(|a, b| b.1.cmp(&a.1));
        } else {
            self.non_captures.sort_by(|a, b| b.1.cmp(&a.1));
        }
    }

    #[inline]
    fn determine_bucket(&self, board: &Board, m: &MoveWithEval) -> BucketType {
        if let MoveDescription::NormalMove(from_coord, to_coord, _) = m.description() {
            if let Square::Occupied(captured_piece, _) = board.get_by_index(to_coord.value()) {
                if let Square::Occupied(attacker_piece, _) = board.get_by_index(from_coord.value()) {
                    let diff = evaluate_piece(*captured_piece) - evaluate_piece(*attacker_piece);
                    if diff > 0 {
                        return BucketType::WinningCapture;
                    } else if diff == 0 {
                        return BucketType::EqualCapture;
                    } else {
                        return BucketType::LosingCapture;
                    }
                }
            }
        }
        BucketType::NonCapture
    }
}
