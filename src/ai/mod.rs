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
    fast_found_hits: usize,
    show_tree_left_side: bool
}

enum SingleMoveResult { NewAlpha(f32), BetaCutOff(f32), NoEffect }

#[derive(Clone)]
enum MemoType { Low, Exact(MoveSnapshot), High(MoveSnapshot) }

#[derive(Clone)]
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
            fast_found_hits: 0,
            show_tree_left_side: false
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
            console_log!("\nBegin depth {}", d);
            self.show_tree_left_side = true;
            self.negamax(d, -MAX_EVAL, MAX_EVAL, 0);

            let leading_move = self.get_leading_move();
            if let Some((m, e)) = leading_move {
                console_log!("{}, {}", m, e);
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
            self.show_tree_left_side = false;
            let eval = evaluation::evaluate(&self.test_board, &mut self.eval_temp_arr);
            return self.test_board.get_player_with_turn().get_multiplier() * eval;
        }

        const NEW_ALPHA_I_NEVER_SET: i32 = -1;
        const NEW_ALPHA_I_HASH_MOVE: i32 = -2;
        let mut new_alpha_i: i32 = NEW_ALPHA_I_NEVER_SET;
        // When `new_alpha_i` is `NEW_ALPHA_I_HASH_MOVE`, the hash move can be found here
        let mut hash_move: Option<MoveSnapshot> = None;

        {
            let memo = {
                let _memo: Option<&MemoData> = self.memo.get(&self.test_board.get_hash());
                _memo.map(|x| x.clone())
            };

            if let Some(MemoData(saved_num, saved_depth, t)) = memo {

                // Using the memo, try to completely avoid any computation for this call
                if saved_depth >= remaining_depth {
                    let r = saved_num;
                    match t {
                        MemoType::Low => {
                            if r <= alpha {
                                self.memo_hits += 1;
                                self.show_tree_left_side = false;
                                return alpha;
                            }
                        },
                        MemoType::High(_) => {
                            if r >= beta { 
                                self.memo_hits += 1;
                                self.show_tree_left_side = false;
                                return beta; 
                            }
                        },
                        MemoType::Exact(_) => {
                            self.memo_hits += 1;
                            self.show_tree_left_side = false;

                            if r < alpha { return alpha; }
                            else if r > beta { return beta; }
                            else { return r; }
                        }
                    };
                }

                // At this point, cannot simply use memoized result.
                // Get PV or refutation move from memo, try it out at full depth before using time on move generation,
                // and either beta cut off or use as candidate-to-beat among rest of moves after move generation.
                let best_move: Option<MoveSnapshot> = match t {
                    MemoType::Exact(m) | MemoType::High(m) => Some(m),
                    _ => None
                };

                if let Some(m) = best_move {

                    if self.show_tree_left_side {
                        crate::console_log!("L = {} (Hash)", m);
                    }

                    let r = unsafe { self.negamax_try_move(
                        remaining_depth, 
                        alpha,
                        false,
                        beta,
                        &m,
                        moves_start
                    ) };
                    match r {
                        SingleMoveResult::BetaCutOff(max_this) => {
                            self.memo.insert(
                                self.test_board.get_hash(),
                                MemoData(max_this, remaining_depth, MemoType::High(m))
                            );
                            self.show_tree_left_side = false;
                            return beta;
                        },
                        SingleMoveResult::NewAlpha(max_this) => {
                            // The move loop below will begin not with the alpha provided from caller,
                            // but with the proven better alpha re-examined at full depth from the memo, which is also an exact score
                            alpha = max_this;
                            new_alpha_i = NEW_ALPHA_I_HASH_MOVE;
                            hash_move = Some(m);
                        },
                        SingleMoveResult::NoEffect => {
                            // The memoized move was not very good after examining it full depth
                        }
                    };
                }
            }
        }

        // Generate moves
        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        // Order by memoized evaluations, then by aggression heuristic
        for i in moves_start..moves_end_exclusive {
            let m = self.moves_buf.get_mutable_snapshot(i);

            self.test_board.handle_move(&m, true);

            let memo: Option<&MemoData> = self.memo.get(&self.test_board.get_hash());

            const BIG_NUMBER: f32 = 100.;
            const EVAL_UPPER_BOUND: f32 = 999.;
            let r = if let Some(MemoData(opponent_max_this, _, MemoType::Exact(_))) = memo {
                -*opponent_max_this * BIG_NUMBER
            } else {
                -EVAL_UPPER_BOUND * BIG_NUMBER
            };

            self.test_board.handle_move(&m, false);
            (*m).1 = r;
        }
        evaluation::sort_moves_by_aggression(
            &self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive, &mut self.eval_temp_arr, &mut self.temp_moves
        );

        if self.show_tree_left_side {
            if new_alpha_i != NEW_ALPHA_I_HASH_MOVE {
                crate::console_log!("L = {}", self.moves_buf.get_v()[moves_end_exclusive - 1]);
            }
        }

        for i in (moves_start..moves_end_exclusive).rev() {

            let r = unsafe { self.negamax_try_move(
                remaining_depth, 
                alpha,
                new_alpha_i != NEW_ALPHA_I_NEVER_SET,
                beta,
                &self.moves_buf.get_v()[i],
                moves_end_exclusive
            )};

            if let SingleMoveResult::NewAlpha(max_this) = r {
                alpha = max_this;
                new_alpha_i = i as i32;
            } else if let SingleMoveResult::BetaCutOff(max_this) = r {
                self.memo.insert(
                    self.test_board.get_hash(),
                    MemoData(max_this, remaining_depth, MemoType::High(self.moves_buf.get_v()[i].clone()))
                );
                self.show_tree_left_side = false;
                return beta;
            }
        }

        if new_alpha_i == NEW_ALPHA_I_HASH_MOVE {
            self.memo.insert(
                self.test_board.get_hash(),
                MemoData(alpha, remaining_depth, MemoType::Exact(hash_move.unwrap()))
            );
        } else if new_alpha_i >= 0 {
            self.memo.insert(
                self.test_board.get_hash(),
                MemoData(alpha, remaining_depth, MemoType::Exact(self.moves_buf.get_v()[new_alpha_i as usize].clone()))
            );
        } else {
            self.memo.insert(self.test_board.get_hash(), MemoData(alpha, remaining_depth, MemoType::Low));
        }
        alpha
    }

    unsafe fn negamax_try_move(
        // Unsafe to allow `m` and `self` be aliases
        &mut self,
        remaining_depth: u8,
        alpha: f32,
        is_alpha_exact_eval: bool,
        beta: f32,
        m: *const MoveSnapshot,
        moves_start: usize
    ) -> SingleMoveResult {
        self.test_board.handle_move(&*m, true);

        let mut fast_found_max_this = 0.0f32;
        let mut fast_found = false;

        if is_alpha_exact_eval {
            // PVS idea - Do a fast boolean check that the current best move with score alpha is really the best.
            // If we always bet correctly, then the second more expensive negamax below is always avoided.
            fast_found_max_this = -self.negamax(remaining_depth - 1, -alpha - 0.01, -alpha, moves_start);
            if fast_found_max_this <= alpha {
                fast_found = true;
                self.fast_found_hits += 1;
            }
        } 

        let max_this = if fast_found {
            fast_found_max_this
        } else {
            -self.negamax(remaining_depth - 1, -beta, -alpha, moves_start)
        };

        self.test_board.handle_move(&*m, false);

        if max_this >= beta {
            SingleMoveResult::BetaCutOff(max_this)
        } else if max_this > alpha {
            SingleMoveResult::NewAlpha(max_this)
        } else {
            SingleMoveResult::NoEffect
        }
    }
}
