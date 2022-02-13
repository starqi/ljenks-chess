mod evaluation;

use std::collections::HashMap;
use super::game::entities::*;
use super::game::move_test::*;
use super::game::move_list::*;
use super::game::board::*;
use super::extern_funcs::now;
use crate::{console_log};

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves: MoveList,
    af_boards: AttackFromBoards,
    memo: HashMap<u64, MemoData>,
    q_memo: HashMap<u64, MemoData>,
    memo_hits: usize,
    fast_found_hits: usize,
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
            af_boards: AttackFromBoards::new(),
            memo: HashMap::new(),
            q_memo: HashMap::new(),
            memo_hits: 0,
            fast_found_hits: 0,
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
            unsafe {
                self.negamax(d, -MAX_EVAL, MAX_EVAL, 0);
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

    fn find_memo_score(&mut self, remaining_depth: i8, alpha: i32, beta: i32) -> (Option<&MemoType>, Option<i32>) {
        if let Some(MemoData(saved_num, saved_depth, memo_type)) = self.memo.get(&self.test_board.get_hash()) {

            // If the memoized move has the precision we want, use its score
            if *saved_depth >= remaining_depth {
                match memo_type {
                    MemoType::Low(_) => {
                        if *saved_num <= alpha {
                            self.memo_hits += 1;
                            return (Some(memo_type), Some(alpha));
                        }
                    },
                    MemoType::High(_) => {
                        if *saved_num >= beta { 
                            self.memo_hits += 1;
                            return (Some(memo_type), Some(beta));
                        }
                    },
                    MemoType::Exact(_) => {
                        self.memo_hits += 1;

                        if *saved_num < alpha {
                            return (Some(memo_type), Some(alpha)); 
                        } else if *saved_num > beta {
                            return (Some(memo_type), Some(beta)); 
                        } else {
                            return (Some(memo_type), Some(*saved_num)); 
                        }
                    }
                };
            }; 

            (Some(memo_type), None)
        } else {
            (None, None)
        }
    }

    fn qsearch(
        &mut self,
        remaining_depth_opt: i8,
        alpha: i32,
        beta: i32,
        moves_start: usize
    ) -> i32 {
        let score = if let (_, Some(adjusted_score)) = self.find_memo_score(0, alpha, beta) {
            adjusted_score
        } else {
            evaluation::evaluate(&self.test_board, &mut self.af_boards)
        };

        return Self::cap(self.test_board.get_player_with_turn().multiplier() * score, alpha, beta);
    }

    /// Will assume ownership over all move list elements from `moves_start`
    /// Only calculates score
    unsafe fn negamax(
        &mut self,
        remaining_depth: i8,
        mut alpha: i32,
        beta: i32,
        moves_start: usize
    ) -> i32 {
        self.node_counter += 1;

        if remaining_depth <= 0 {

            return self.qsearch(10, alpha, beta, moves_start);
            /*
            if quiescence {
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

        const NEW_ALPHA_I_NEVER_SET: i32 = -1;
        const NEW_ALPHA_I_HASH_MOVE: i32 = -2;

        let mut new_alpha_i: i32 = NEW_ALPHA_I_NEVER_SET;
        // When `new_alpha_i` is `NEW_ALPHA_I_HASH_MOVE`, the hash move can be found here
        let mut hash_move: Option<MoveWithEval> = None;

        match self.find_memo_score(remaining_depth, alpha, beta) {
            (_, Some(adjusted_score)) => { // Use memoized move
                return adjusted_score;
            },
            (Some(memo_type), None) => { // Memoized move is not precise enough, try using it as the first best move

                // Clone the move, unlike if the move is on move list, because memo updates will overwrite the same memory. Also, we can move it into the memo.
                let move_clone = if let MemoType::Exact(m) | MemoType::High(m) = memo_type {
                    Some(m.clone())
                } else {
                    // (1) For fail low memo entries, currently the move is a random move so we can't use it as the best move (but it doesn't have to be that way TODO)
                    None
                };

                if let Some(m) = move_clone {
                    match self.negamax_try_move(remaining_depth, alpha, true, beta, &m, moves_start) {
                        SingleMoveResult::BetaCutOff(score) => {
                            self.memo.insert(
                                self.test_board.get_hash(),
                                MemoData(score, remaining_depth, MemoType::High(m))
                            );
                            return beta;
                        },
                        SingleMoveResult::NewAlpha(score) => {
                            // The move loop below will begin not with the alpha provided from caller,
                            // but with the proven better alpha re-examined at full depth from the memo, which is also an exact score.
                            alpha = score;
                            new_alpha_i = NEW_ALPHA_I_HASH_MOVE;
                            hash_move = Some(m);
                        },
                        SingleMoveResult::NoEffect => {
                            // The memoized move was not very good after examining it full depth, begin normal loop through moves.
                        }
                    };
                }
            },
            _ => {}
        };

        // Generate moves and order
        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;
        for i in moves_start..moves_end_exclusive {

            let m = self.moves_buf.get_mutable_snapshot(i);
            let revertable = self.test_board.handle_move(&m);
            let memo: Option<&MemoData> = self.memo.get(&self.test_board.get_hash());

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

        if moves_start == moves_end_exclusive {
            return self.get_no_moves_eval(alpha, beta);
        }

        //FIXME
        //evaluation::add_aggression_to_evals(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive, &mut self.temp_moves);
        evaluation::add_captures_to_evals(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive);
        self.moves_buf.sort_subset_by_eval(moves_start, moves_end_exclusive);

        for i in (moves_start..moves_end_exclusive).rev() {
            let m: *const MoveWithEval = &self.moves_buf.v()[i];

            //FIXME
            //let diff = i - moves_start;
            //let less_depth_amount = min((-((diff >= 5) as i8) as usize) & (diff >> 2), 2) as i8;
            //let less_depth_amount = ((-((diff >= 5) as i8) as usize) & 1) as i8;
            let less_depth_amount = 0;
            
            let r = self.negamax_try_move(
                remaining_depth - less_depth_amount, 
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
                self.memo.insert(
                    self.test_board.get_hash(),
                    MemoData(max_this, remaining_depth, MemoType::High((*m).clone()))
                );
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
                MemoData(alpha, remaining_depth, MemoType::Exact(self.moves_buf.v()[new_alpha_i as usize].clone()))
            );
        } else {
            // See (1)
            self.memo.insert(
                self.test_board.get_hash(),
                MemoData(alpha, remaining_depth, MemoType::Low(self.moves_buf.v()[0].clone()))
            );
        }
        alpha
    }

    /// Unsafe purpose: allow reference to `m` while the move list holding it is being mutated, trusting proper management of move list subsets.
    unsafe fn negamax_try_move(
        &mut self,
        remaining_depth: i8,
        alpha: i32,
        is_alpha_exact_eval: bool,
        beta: i32,
        m: *const MoveWithEval,
        moves_start: usize
    ) -> SingleMoveResult {
        let revertable = self.test_board.handle_move(&*m);

        let mut fast_found_max_this: i32 = 0;
        let mut fast_found = false;

        if is_alpha_exact_eval {

            // PVS intuition - Bet on the current best move staying the best move. If it's a good bet,
            // normal alpha-beta would likely fail low - alpha is not incremented - so for speed, we move up the -beta argument
            // to as close to alpha as possible -alpha - 1, for a faster/narrower window, and this doesn't change the result
            // as long as the best move stays the best. 
            // If wrong, then we need to do another full search to satisfy the contract of providing a proper score between alpha and beta.

            fast_found_max_this = -self.negamax(remaining_depth - 1, -alpha - 1, -alpha, moves_start);
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
