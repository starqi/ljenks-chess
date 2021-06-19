mod evaluation;

use std::collections::HashMap;
use super::game::move_list::*;
use super::game::board::*;
use crate::{console_log};

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves: MoveList,
    eval_temp_arr: [f32; 64],
    memo: HashMap<u64, MemoData>,
    memo_hits: usize,
    fast_found_hits: usize
}

enum MemoType { Low, Exact(MoveSnapshot), High(MoveSnapshot) }
struct MemoData(f32, u8, MemoType);

static MAX_EVAL: f32 = 9000.;

impl Ai {

    pub fn new() -> Self {
        console_log!("AI init");
        Self {
            moves_buf: MoveList::new(1000),
            test_board: Board::new(),
            temp_moves: MoveList::new(50),
            eval_temp_arr: [0.; 64],
            memo: HashMap::new(),
            memo_hits: 0,
            fast_found_hits: 0
        }
    }

    fn get_leading_move(&self) -> Option<(&MoveSnapshot, f32)> {
        match self.memo.get(&self.test_board.get_hash()) {
            Some(MemoData(eval, _, MemoType::Exact(best_move))) => {
                Some((best_move, *eval))
            },
            _ => {
                None
            }
        }
    }

    pub fn make_move(&mut self, depth: u8, real_board: &mut Board) {

        self.test_board.clone_from(real_board);

        for d in (1..=depth).step_by(2) {
            self.moves_buf.write_index = 0;
            self.negamax(d, -MAX_EVAL, MAX_EVAL, 0);
            let leading_move = self.get_leading_move();
            if let Some((m, e)) = leading_move {
                console_log!("Depth {}, {}, {}", d, m, e);
            } else {
                console_log!("No leading move");
            }
        }

        let c_hash = self.test_board.calculate_hash();
        console_log!("DEBUG Hash {} vs. {}? {}", self.test_board.get_hash(), c_hash, c_hash == self.test_board.get_hash());

        let leading_move = self.get_leading_move();
        if let Some((m, e)) = leading_move {
            console_log!("Making move: {} ({})", m, e);
            real_board.handle_move(m, true);
        } else {
            console_log!("No move");
        }
        console_log!("Memo hits - {}, size - {}, fast found - {}", self.memo_hits, self.memo.len(), self.fast_found_hits);
        self.memo_hits = 0;
        self.fast_found_hits = 0;
        self.memo.clear();
    }

    /// Will assume ownership over all move list elements from `moves_start`
    /// Only calculates score
    fn negamax(
        &mut self,
        remaining_depth: u8,
        mut alpha: f32,
        beta: f32,
        moves_start: usize
    ) -> f32 {

        if remaining_depth <= 0 {
            let eval = evaluation::evaluate(&self.test_board, &mut self.eval_temp_arr);
            return self.test_board.get_player_with_turn().get_multiplier() * eval;
        }

        let memo: Option<&MemoData> = self.memo.get(&self.test_board.get_hash());
        if let Some(MemoData(saved_num, saved_depth, t)) = memo {
            if *saved_depth >= remaining_depth {
                let r = *saved_num;
                match t {
                    MemoType::Low => {
                        if r <= alpha {
                            self.memo_hits += 1;
                            return alpha;
                        }
                    },
                    MemoType::High(_) => {
                        if r >= beta { 
                            self.memo_hits += 1;
                            return beta; 
                        }
                    },
                    MemoType::Exact(_) => {
                        self.memo_hits += 1;
                        if r < alpha { return alpha; }
                        else if r > beta { return beta; }
                        else { return r; }
                    }
                };
            }
        }

        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        for i in moves_start..moves_end_exclusive {
            let m = self.moves_buf.get_mutable_snapshot(i);

            self.test_board.handle_move(&m, true);

            let memo: Option<&MemoData> = self.memo.get(&self.test_board.get_hash());

            // Prioritize memo evals over aggression
            const BIG_NUMBER: f32 = 100.;
            let r = if let Some(MemoData(max_this, _, MemoType::Exact(_))) = memo {
                *max_this * BIG_NUMBER
            } else {
                -999. * BIG_NUMBER
            };

            self.test_board.handle_move(&m, false);
            (*m).1 = r;
        }
        evaluation::sort_moves_by_aggression(
            &self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive, &mut self.eval_temp_arr, &mut self.temp_moves
        );

        let mut new_alpha_i: i32 = -1;
        for i in (moves_start..moves_end_exclusive).rev() {

            self.test_board.handle_move(&self.moves_buf.get_v()[i], true);

            let mut fast_found_max_this = 0.0f32;
            let mut fast_found = false;

            if new_alpha_i >= 0 {
                fast_found_max_this = -self.negamax(remaining_depth - 1, -alpha - 0.01, -alpha, moves_end_exclusive);
                if fast_found_max_this <= alpha {
                    fast_found = true;
                    self.fast_found_hits += 1;
                }
            } 

            let max_this = if fast_found {
                fast_found_max_this
            } else {
                -self.negamax(remaining_depth - 1, -beta, -alpha, moves_end_exclusive)
            };

            self.test_board.handle_move(&self.moves_buf.get_v()[i], false);

            if max_this >= beta {
                self.memo.insert(
                    self.test_board.get_hash(),
                    MemoData(max_this, remaining_depth, MemoType::High(self.moves_buf.get_v()[i as usize].clone()))
                );
                return beta;
            } else if max_this > alpha {
                alpha = max_this;
                new_alpha_i = i as i32;
            }
        }

        if new_alpha_i >= 0 {
            self.memo.insert(
                self.test_board.get_hash(),
                MemoData(alpha, remaining_depth, MemoType::Exact(self.moves_buf.get_v()[new_alpha_i as usize].clone()))
            );
        } else {
            self.memo.insert(self.test_board.get_hash(), MemoData(alpha, remaining_depth, MemoType::Low));
        }
        alpha
    }
}
