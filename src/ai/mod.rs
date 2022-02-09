mod evaluation;

use std::collections::HashMap;
use super::game::entities::*;
use super::game::move_list::*;
use super::game::board::*;
use super::extern_funcs::now;
use crate::{console_log};
use evaluation::ValueBoard;

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves: MoveList,
    temp_value_board: ValueBoard,
    memo: HashMap<u64, MemoData>,
    q_memo: HashMap<u64, MemoData>,
    memo_hits: usize,
    fast_found_hits: usize,
    show_tree_left_side: bool,
    node_counter: u32
}

enum SingleMoveResult { NewAlpha(i32), BetaCutOff(i32), NoEffect }

#[derive(Clone)]
enum MemoType { Low(MoveWithEval), Exact(MoveWithEval), High(MoveWithEval) }

#[derive(Clone)]
struct MemoData(i32, i8, MemoType);

static MAX_EVAL: i32 = 999999;

impl Ai {

    pub fn new() -> Self {
        console_log!("AI init");
        Self {
            moves_buf: MoveList::new(1000),
            test_board: Board::new(),
            temp_moves: MoveList::new(50),
            temp_value_board: ValueBoard::new(),
            memo: HashMap::new(),
            q_memo: HashMap::new(),
            memo_hits: 0,
            fast_found_hits: 0,
            show_tree_left_side: false,
            node_counter: 0
        }
    }

    fn get_leading_move(&self) -> Option<(&MoveWithEval, i32)> {
        match self.memo.get(&self.test_board.get_hash()) {
            // In this context, fail high means checkmate
            Some(MemoData(eval, _, MemoType::High(best_move) | MemoType::Exact(best_move) | MemoType::Low(best_move))) => {
                Some((best_move, *eval))
            },
            _ => {
                None
            }
        }
    }

    pub fn make_move(&mut self, depth: i8, real_board: &mut Board) {

        self.test_board.clone_from(real_board);

        let start_ms = now();
        for d in (1i8..=depth).step_by(2) {
            console_log!("\nBegin depth {}", d);
            self.show_tree_left_side = true;
            unsafe {
                self.negamax(d, false, -MAX_EVAL, MAX_EVAL, 0);
            }

            let leading_move = self.get_leading_move();
            if let Some((m, e)) = leading_move {
                console_log!("{}, {}", self.test_board.stringify_move(m), e);
            } else {
                console_log!("No leading move");
            }
        }

        self.test_board.assert_hash();
        self.assert_king_pos(Player::White);
        self.assert_king_pos(Player::Black);

        let leading_move = self.get_leading_move();
        if let Some((m, e)) = leading_move {
            console_log!("Making move: {} ({})", self.test_board.stringify_move(m), e);
            real_board.handle_move(m);
        } else {
            console_log!("No move");
        }
        console_log!("Memo hits - {}, size - {} / q - {}, fast found - {}", self.memo_hits, self.memo.len(), self.q_memo.len(), self.fast_found_hits);
        console_log!("NPS - {}", (self.node_counter as f64 / ((now() - start_ms) as f64 / 1000.)).round());

        self.node_counter = 0;
        self.memo_hits = 0;
        self.fast_found_hits = 0;
        self.memo.clear();
        self.q_memo.clear();
    }

    fn assert_king_pos(&self, player: Player) {
        if let Square::Occupied(Piece::King, player) = self.test_board.get_by_index(self.test_board.get_player_state(player).king_location._lsb_to_index()) {
        } else {
            panic!("Wrong king square detected for {:?}", player);
        }
    }

    /// Will assume ownership over all move list elements from `moves_start`
    /// Only calculates score
    unsafe fn negamax(
        &mut self,
        remaining_depth: i8,
        quiescence: bool,
        mut alpha: i32,
        beta: i32,
        moves_start: usize
    ) -> i32 {
        self.node_counter += 1;

        if remaining_depth <= 0 {
            self.show_tree_left_side = false;

            self.temp_moves.write_index = 0;
            let eval = evaluation::evaluate(&self.test_board, &mut self.temp_moves, &mut self.temp_value_board);

            return Self::cap(self.test_board.get_player_with_turn().multiplier() * eval, alpha, beta);


            /*
            if quiescence {
                self.show_tree_left_side = false;
                let eval = evaluation::evaluate(&self.test_board, &mut self.eval_temp_arr);
                return Self::cap(self.test_board.get_player_with_turn().multiplier() * eval, alpha, beta);
            } else {
                let mut eval = evaluation::evaluate(&self.test_board, &mut self.eval_temp_arr);
                eval = self.test_board.get_player_with_turn().multiplier() * eval;

                // Typical quiescence pruning (TODO review)
                if eval >= beta { return beta; }
                if eval > alpha { alpha = eval; }

                return self.negamax(3, true, alpha, beta, moves_start);
            }
            */
        }

        let resolved_memo: *mut HashMap<u64, MemoData> = if quiescence { &mut self.q_memo } else { &mut self.memo };

        const NEW_ALPHA_I_NEVER_SET: i32 = -1;
        const NEW_ALPHA_I_HASH_MOVE: i32 = -2;
        let mut new_alpha_i: i32 = NEW_ALPHA_I_NEVER_SET;
        // When `new_alpha_i` is `NEW_ALPHA_I_HASH_MOVE`, the hash move can be found here
        let mut hash_move: Option<MoveWithEval> = None;

        {
            let memo = {
                let _memo: Option<&MemoData> = (*resolved_memo).get(&self.test_board.get_hash());
                _memo.map(|x| x.clone())
            };

            if let Some(MemoData(saved_num, saved_depth, t)) = memo {

                // Using the memo, try to completely avoid any computation for this call
                if saved_depth >= remaining_depth {
                    let r = saved_num;
                    match t {
                        MemoType::Low(_) => {
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
                // Get PV or refutation move from memo, try it out at full depth before computing move generation,
                // and either beta cut off or use as candidate-to-beat among rest of moves after move generation.
                let best_move: Option<MoveWithEval> = match t {
                    MemoType::Exact(m) | MemoType::High(m) => Some(m),
                    _ => None
                };

                if let Some(m) = best_move {

                    let run = if quiescence {
                        self.is_unstable_move(&m)
                    } else {
                        true
                    };

                    if run {
                        if self.show_tree_left_side {
                            console_log!("L = {} (Hash) {}", self.test_board.stringify_move(&m), if quiescence { "(Q)" } else { "" });
                        }

                        let r = self.negamax_try_move(
                            remaining_depth, 
                            quiescence,
                            alpha,
                            false,
                            beta,
                            &m,
                            moves_start
                        );

                        match r {
                            SingleMoveResult::BetaCutOff(max_this) => {
                                (*resolved_memo).insert(
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
        }

        // Generate moves
        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        // Order by memoized evaluations, then by aggression heuristic
        for i in moves_start..moves_end_exclusive {

            let m = self.moves_buf.get_mutable_snapshot(i);
            let revertable = self.test_board.handle_move(&m);
            let memo: Option<&MemoData> = (*resolved_memo).get(&self.test_board.get_hash());

            const BIG_NUMBER: i32 = 10000;
            const EVAL_UPPER_BOUND: i32 = 99999;
            let r = if let Some(MemoData(opponent_max_this, _, MemoType::Exact(_))) = memo {
                -*opponent_max_this * BIG_NUMBER
            } else {
                -EVAL_UPPER_BOUND * BIG_NUMBER
            };

            self.test_board.revert_move(&revertable);
            (*m).1 = r;
        }

        if !quiescence {
            // No moves for non-quiescence
            if moves_start == moves_end_exclusive {
                self.show_tree_left_side = false;
                return self.get_no_moves_eval(alpha, beta);
            }
            //evaluation::add_aggression_to_evals(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive, &mut self.temp_moves);
        }
        evaluation::add_captures_to_evals(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive);
        self.moves_buf.sort_subset_by_eval(moves_start, moves_end_exclusive);

        if self.show_tree_left_side {
            if new_alpha_i != NEW_ALPHA_I_HASH_MOVE {
                if !quiescence {
                    console_log!("L = {}", self.test_board.stringify_move(&self.moves_buf.v()[moves_end_exclusive - 1]));
                }
            }
        }

        let mut has_quiescence_move = false;
        for i in (moves_start..moves_end_exclusive).rev() {
            let m: *const MoveWithEval = &self.moves_buf.v()[i];

            if quiescence {
                if !self.is_unstable_move(&*m) { continue; }
                if !has_quiescence_move && self.show_tree_left_side {
                    if new_alpha_i != NEW_ALPHA_I_HASH_MOVE {
                        console_log!("L = {} (Quiescence)", self.test_board.stringify_move(&*m));
                        console_log!("{}", self.test_board);
                    }
                }
                has_quiescence_move = true;
            }

            let diff = i - moves_start;
            //let less_depth_amount = min((-((diff >= 5) as i8) as usize) & (diff >> 2), 2) as i8;
            let less_depth_amount = ((-((diff >= 5) as i8) as usize) & 1) as i8;
            
            let r = self.negamax_try_move(
                remaining_depth - less_depth_amount, 
                quiescence,
                alpha,
                new_alpha_i != NEW_ALPHA_I_NEVER_SET,
                beta,
                m,
                moves_end_exclusive
            );

            if let SingleMoveResult::NewAlpha(max_this) = r {
                alpha = max_this;
                new_alpha_i = i as i32;
            } else if let SingleMoveResult::BetaCutOff(max_this) = r {
                (*resolved_memo).insert(
                    self.test_board.get_hash(),
                    MemoData(max_this, remaining_depth, MemoType::High((*m).clone()))
                );
                self.show_tree_left_side = false;
                return beta;
            }
        }
        if quiescence && !has_quiescence_move {
            // No moves for quiescence
            self.show_tree_left_side = false;
            return alpha;
        }

        if new_alpha_i == NEW_ALPHA_I_HASH_MOVE {
            (*resolved_memo).insert(
                self.test_board.get_hash(),
                MemoData(alpha, remaining_depth, MemoType::Exact(hash_move.unwrap()))
            );
        } else if new_alpha_i >= 0 {
            (*resolved_memo).insert(
                self.test_board.get_hash(),
                MemoData(alpha, remaining_depth, MemoType::Exact(self.moves_buf.v()[new_alpha_i as usize].clone()))
            );
        } else {
            (*resolved_memo).insert(
                self.test_board.get_hash(),
                MemoData(alpha, remaining_depth, MemoType::Low(self.moves_buf.v()[0].clone()))
            );
        }
        alpha
    }

    unsafe fn negamax_try_move(
        // Unsafe to allow `m` and `self` be aliases
        &mut self,
        remaining_depth: i8,
        quiescence: bool,
        alpha: i32,
        is_alpha_exact_eval: bool,
        beta: i32,
        m: *const MoveWithEval,
        moves_start: usize
    ) -> SingleMoveResult {
        let revertable = self.test_board.handle_move(&*m);

        let mut fast_found_max_this: i32 = 0;
        let mut fast_found = false;

        if !quiescence && is_alpha_exact_eval {
            // PVS idea - Do a fast boolean check that the current best move with score alpha is really the best.
            // If we always bet correctly, then the second more expensive negamax below is always avoided.
            fast_found_max_this = -self.negamax(remaining_depth - 1, false, -alpha - 1, -alpha, moves_start);
            if fast_found_max_this <= alpha {
                fast_found = true;
                self.fast_found_hits += 1;
            }
        } 

        let max_this = if fast_found {
            fast_found_max_this
        } else {
            -self.negamax(remaining_depth - 1, quiescence, -beta, -alpha, moves_start)
        };

        self.test_board.revert_move(&revertable);

        if max_this >= beta {
            SingleMoveResult::BetaCutOff(max_this)
        } else if max_this > alpha {
            SingleMoveResult::NewAlpha(max_this)
        } else {
            SingleMoveResult::NoEffect
        }
    }

    fn is_unstable_move(&self, m: &MoveWithEval) -> bool {
        self.test_board.is_capture(m)
    }

    fn cap(r: i32, alpha: i32, beta: i32) -> i32 {
        if r <= alpha { return alpha; }
        else if r >= beta { return beta; }
        else { return r; }
    }

    fn get_no_moves_eval(&mut self, alpha: i32, beta: i32) -> i32 {
        let checking_player = self.test_board.get_player_with_turn().other_player();
        if self.test_board.is_checking(checking_player) {
            return alpha;
        } else {
            return Self::cap(0, alpha, beta);
        }
    }
}
