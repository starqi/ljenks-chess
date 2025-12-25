mod evaluation;

use std::cmp::min;
use std::collections::HashMap;
use super::game::entities::*;
use super::game::move_test::*;
use super::game::move_list::*;
use super::game::board::*;
use super::extern_funcs::{random, now};
use crate::game::stringify::slow_stringify_move_standard;
use crate::{console_log};

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves: MoveList,
    af_boards: AttackFromBoards,
    memo: HashMap<u64, MemoData>,
    useful_memo_hits: usize,
    hash_move_memo_hits: usize,
    fast_found_hits: usize,
    node_counter: u64,
    start_ms: u128,
    ms_till_terminate: u128, // Configuration
    terminated: bool,
    depth: i8, // Configuration
    iterative_deepening_depth: i8,
    real_board_positional_hashes: *const Vec<u64>
}

// TODO (Minor) Rename this, NoEffect -> Fail low
enum SingleMoveResult { NewAlpha(i32), BetaCutOff(i32), NoEffect }

// TODO Refactor these 2 classes, Clone on enum?

/// Stored move and eval is the best move at the memo position.
#[derive(Clone)]
enum MemoType { LessThan(MoveWithEval), Exact(MoveWithEval), GreaterThan(MoveWithEval) }

/// (Score which can be exact or lower or upper bound depending on type, remaining depth, type, age)
#[derive(Clone)]
struct MemoData(i32, i8, MemoType, usize);

static MAX_EVAL: i32 = 999999;

static RANDOMIZATION_DIFF: i32 = 20; // Too high (50) -> game never ends from weird stall moves...?

impl Ai {

    /// Must be called, second "new". Had issues with wasm tracking Rust lifetimes, and couldn't do it in constructor...
    pub fn late_inject(&mut self, real_board_positional_hashes: &Vec<u64>) {
        self.real_board_positional_hashes = real_board_positional_hashes;
    }

    pub fn new() -> Self {
        console_log!("AI init");
        Self {
            moves_buf: MoveList::new(1000),
            test_board: Board::new(),
            temp_moves: MoveList::new(50),
            af_boards: AttackFromBoards::new(),
            memo: HashMap::new(),
            useful_memo_hits: 0,
            hash_move_memo_hits: 0,
            fast_found_hits: 0,
            node_counter: 0,
            start_ms: 0,
            ms_till_terminate: 1000,
            terminated: false,
            depth: 5,
            iterative_deepening_depth: 1,
            real_board_positional_hashes: 0 as *const Vec<u64>
        }
    }

    fn get_leading_move(&self) -> Option<(&MoveWithEval, i8)> {
        match self.memo.get(&self.test_board.get_hash()) {
            // In this context, fail high means checkmate
            Some(MemoData(_, remaining_depth, MemoType::GreaterThan(best_move) | MemoType::Exact(best_move) | MemoType::LessThan(best_move), _)) => {
                Some((best_move, *remaining_depth))
            },
            _ => {
                None
            }
        }
    }

    pub fn make_move(&mut self, real_board: &mut Board) -> Option<String> {

        self.test_board.clone_from(real_board);

        self.start_ms = now();
        self.terminated = false;
        for d in (1i8..=self.depth).step_by(2) {
            console_log!("\nBegin depth {}", d);
            self.iterative_deepening_depth = d;
            unsafe {
                self.negamax(d, -MAX_EVAL, MAX_EVAL, 0);
            }

            let leading_move = self.get_leading_move();
            if let Some((m, depth)) = leading_move {
                console_log!("{}, d={}", self.test_board.stringify_move_for_js_logs(m), depth);
            } else {
                console_log!("No leading move");
            }

            if self.terminated { 
                console_log!("Terminated due to time");
                break; 
            }
        }

        self.test_board.assert_hash();
        self.assert_king_pos(Player::White);
        self.assert_king_pos(Player::Black);

        let leading_move = self.get_leading_move();
        let notation = if let Some((best_move, remaining_depth)) = leading_move {
            console_log!("Making move: {} (depth = {})", self.test_board.stringify_move_for_js_logs(best_move), remaining_depth);

            let before_info = BeforeMoveInfoForStringify::slow_new(real_board, best_move);
            let original_player = real_board.get_player_with_turn();

            real_board.handle_move_no_revert(best_move);

            let is_check = real_board.is_checking(original_player);
            let is_checkmate = is_check && real_board.has_no_legal_moves();
            let after_info = AfterMoveInfoForStringify { is_check, is_checkmate };

            Some(slow_stringify_move_standard(best_move, &before_info, &after_info))
        } else {
            console_log!("No move");
            None
        };
        console_log!(
            "Useful memo hits - {}, hash move memo hits - {}, size - {}, fast found - {}",
            self.useful_memo_hits,
            self.hash_move_memo_hits,
            self.memo.len(),
            self.fast_found_hits
        );
        console_log!("Nodes - {}, NPS - {}", self.node_counter, (self.node_counter as f64 / ((now() - self.start_ms) as f64 / 1000.)).round());

        self.node_counter = 0;
        self.useful_memo_hits = 0;
        self.hash_move_memo_hits = 0;
        self.fast_found_hits = 0;
        // TODO Memo experiment... statistics...
        //self.memo.clear();
        console_log!("Memo aging, before size = {}", self.memo.len());
        for (_, MemoData(_, _, _, age)) in self.memo.iter_mut() {
            *age += 1;
        }
        self.memo.retain(|_, MemoData(_, _, _, age)| *age <= 5);
        console_log!("Memo aging, after size = {}", self.memo.len());

        notation
    }

    fn assert_king_pos(&self, player: Player) {
        if let Square::Occupied(Piece::King, _) = self.test_board.get_by_index(self.test_board.get_player_state(player).king_location._lsb_to_index()) {
        } else {
            panic!("Wrong king square detected for {:?}", player);
        }
    }

    fn get_no_moves_eval(&mut self, alpha: i32, beta: i32) -> i32 {
        let checking_player = self.test_board.get_player_with_turn().other_player();
        if self.test_board.is_checking(checking_player) {
            return alpha;
        } else {
            if 0 <= alpha { return alpha; }
            else if 0 >= beta { return beta; }
            else { return 0; }
        }
    }

    #[inline]
    fn insert_memo(&mut self, memo_data: MemoData) {
        self.memo.insert(self.test_board.get_hash(), memo_data);
    }

    /// Node counter increase coupled with check to not miss an increment
    fn increment_node_check_termination(&mut self) -> bool {
        self.node_counter += 1;
        self.terminated = self.terminated || (self.node_counter % 50000 == 0 && now() - self.start_ms > self.ms_till_terminate);
        self.terminated
    }

    // TODO Refactor into struct
    /// First tuple entry = the memoized result if any, like an existence boolean
    /// Second tuple entry = exists only if all info is present, e.g. if memo says score >= 5.0, and 
    /// and alpha, beta = 3.0, 6.0 (a constraint range), then cannot return an accurate answer -- 
    /// if real score is 4.0 or 4.2, not enough info stored to know. But if memo says >= 7.0, then all real scores -> 6.0.
    fn find_memo_score(&mut self, remaining_depth: i8, alpha: i32, beta: i32) -> (Option<&MoveWithEval>, Option<i32>) {
        if let Some(MemoData(saved_num, saved_depth, memo_type, _)) = self.memo.get(&self.test_board.get_hash()) {
            // If the memoized move has the precision we want, use its score
            match memo_type {
                MemoType::LessThan(m) => {
                    if *saved_depth >= remaining_depth && *saved_num <= alpha {
                        (Some(m), Some(alpha))
                    } else {
                        (Some(m), None)
                    }
                },
                MemoType::GreaterThan(m) => {
                    if *saved_depth >= remaining_depth && *saved_num >= beta { 
                        (Some(m), Some(beta))
                    } else {
                        (Some(m), None)
                    }
                },
                MemoType::Exact(m) => {
                    if *saved_depth >= remaining_depth {
                        if *saved_num < alpha {
                            (Some(m), Some(alpha))
                        } else if *saved_num > beta {
                            (Some(m), Some(beta))
                        } else {
                            (Some(m), Some(*saved_num))
                        }
                    } else {
                        (Some(m), None)
                    }
                }
            }
        } else {
            (None, None)
        }
    }

    unsafe fn qsearch(
        &mut self,
        remaining_depth_opt: i8,
        initial_alpha: i32,
        beta: i32,
        moves_start: usize
    ) -> i32 {

        // Evaluation is always maximizing for white. Black is also maximizing, so whenever it's black's turn, black's 'score definition" is negative of white's score definition.
        let score_multiplier = self.test_board.get_player_with_turn().multiplier();

        if self.increment_node_check_termination() { return initial_alpha; } // See (2) FIXME WTF IS (2)

        let mut alpha = initial_alpha;

        let score = if let (_, Some(adjusted_score)) = self.find_memo_score(0, alpha, beta) {
            self.useful_memo_hits += 1;
            return adjusted_score; // No score multiplier necessary
        } else {
            score_multiplier * evaluation::evaluate(&self.test_board, &mut self.af_boards)
        };

        if remaining_depth_opt <= 0 { return score; }

        // Intuition: If static evaluation is >= beta, and pretending zugzwang doesn't apply, we stop searching assuming 
        // more free moves will make score go even higher, despite unstable captures still existing.
        if score >= beta { return beta; }

        // Intuition: Same as beta, if static eval is X, given a free move, we expect it to be > X. Of course, if this assumption is wrong and score < X,
        // then we are lying in that the returned score is exact because above initial alpha, but it's not true.
        if score > alpha { alpha = score; }

        // Generate moves and order
        self.moves_buf.write_index = moves_start;
        self.test_board.get_checks_captures_for(self.test_board.get_player_with_turn(), &mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;
        if moves_start == moves_end_exclusive { return score; }

        evaluation::add_captures_to_evals(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive);
        self.moves_buf.sort_subset_by_eval(moves_start, moves_end_exclusive);

        for i in (moves_start..moves_end_exclusive).rev() {
            let m: *const MoveWithEval = &self.moves_buf.v()[i];
            let mut revertable = RevertableMove::NoOp(0);
            self.test_board.handle_move(&*m, &mut revertable);

            let r = -self.qsearch(
                remaining_depth_opt - 1, 
                -beta,
                -alpha,
                moves_end_exclusive
            );

            self.test_board.revert_move(&revertable);

            if r >= beta { 
                if self.terminated { return initial_alpha; } // See (2)
                return beta; 
            }
            if r > alpha { alpha = score; }
        }

        alpha
    }

    /// Will assume ownership over all move list elements from `moves_start`
    /// Only calculates score
    unsafe fn negamax(
        &mut self,
        remaining_depth: i8,
        initial_alpha: i32,
        beta: i32,
        moves_start: usize
    ) -> i32 {
        const NEW_ALPHA_I_NEVER_SET: i32 = -1;
        const NEW_ALPHA_I_HASH_MOVE: i32 = -2;

        if remaining_depth <= 0 {
            return self.qsearch(10, initial_alpha, beta, moves_start);
        }
        if self.increment_node_check_termination() { return initial_alpha; } // Chain force beta cutoff in all parents; checked by assertions (2)

        let mut alpha = initial_alpha;
        let mut new_alpha_i: i32 = NEW_ALPHA_I_NEVER_SET;
        // When `new_alpha_i` is `NEW_ALPHA_I_HASH_MOVE`, the hash move can be found here
        let mut hash_move: Option<MoveWithEval> = None;

        // Shape: (Move with eval, clamped score, is exact match)
        let memo_move_suggestion: Option<(MoveWithEval, i32, bool)> = match self.find_memo_score(remaining_depth, alpha, beta) {
            (Some(m), Some(clamped_score)) => { // Use memoized move
                Some((m.clone(), clamped_score, true))
            },
            (Some(m), None) => {
                // Memoized move is not precise enough, try using it as the first best move.
                // Must clone the move, because recursive memo updates touch the same memory.
                Some((m.clone(), 0, false))
            },
            _ => None
        };

        if let Some((m, clamped_score, true)) = memo_move_suggestion {
            self.useful_memo_hits += 1;

            if remaining_depth >= self.iterative_deepening_depth {
                let mut found_repetition = false;

                let mut revertable = RevertableMove::NoOp(0);
                let before_move_hash = self.test_board.get_hash();
                self.test_board.handle_move(&m, &mut revertable);

                let current_hash = self.test_board.get_hash();
                let unrawed: &Vec<u64> = &*self.real_board_positional_hashes;
                let repetition_count = unrawed.iter().filter(|&&h| h == current_hash).count();
                console_log!("??? reps={}, real#={} remaining_depth_no_lmr={} it={} {}", repetition_count, unrawed.len(), remaining_depth, self.iterative_deepening_depth, unrawed as *const Vec<u64> as usize);
                if repetition_count >= 2 {
                    console_log!("??? GREATER 2");
                    console_log!("HASH THERE 1??? {}", self.memo.contains_key(&before_move_hash));
                    self.memo.remove(&before_move_hash);
                    console_log!("HASH THERE 2??? {}", self.memo.contains_key(&before_move_hash));
                    found_repetition = true;
                }

                if !found_repetition {
                    self.moves_buf.write_index = moves_start;
                    self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
                    let moves_end_exclusive = self.moves_buf.write_index;

                    for i in (moves_start..moves_end_exclusive).rev() {
                        let m: *const MoveWithEval = &self.moves_buf.v()[i];

                        let mut revertable = RevertableMove::NoOp(0);
                        self.test_board.handle_move(&*m, &mut revertable);
                        let current_hash = self.test_board.get_hash();
                        let repetition_count = unrawed.iter().filter(|&&h| h == current_hash).count();
                        if repetition_count >= 2 {
                            console_log!("???2 GREATER 2");
                            console_log!("HASH THERE 1???2 {}", self.memo.contains_key(&before_move_hash));
                            self.memo.remove(&before_move_hash);
                            console_log!("HASH THERE 2???2 {}", self.memo.contains_key(&before_move_hash));
                            found_repetition = true;
                            self.test_board.revert_move(&revertable);
                            break;
                        } else {
                            self.test_board.revert_move(&revertable);
                        }
                    }
                }
                self.test_board.revert_move(&revertable);

                if !found_repetition { return clamped_score; }
            } else {
                return clamped_score;
            }
        } else if let Some((m, _, false)) = memo_move_suggestion {

            // Reminder: No null window, because this is our best move candidate, hence it is not expected to fail low
            match self.negamax_try_move(remaining_depth, remaining_depth, alpha, false, beta, &m, moves_start) {
                SingleMoveResult::BetaCutOff(score) => {
                    self.hash_move_memo_hits += 1;
                    if self.terminated {
                        return initial_alpha; // See (2)
                    } else {
                        self.insert_memo(MemoData(score, remaining_depth, MemoType::GreaterThan(m), 0));
                        return beta;
                    }
                },
                SingleMoveResult::NewAlpha(score) => {
                    self.hash_move_memo_hits += 1;
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
            assert!(!self.terminated);
        }

        // Generate moves and order
        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        // FIXME
        /*
        for i in moves_start..moves_end_exclusive {

            let m = self.moves_buf.get_mutable_snapshot(i);
            let revertable = self.test_board.handle_move(&m);
            let memo: Option<&MemoData> = self.memo.get(&self.test_board.get_hash());

            const BIG_NUMBER: i32 = 10000;
            const EVAL_UPPER_BOUND: i32 = 99999;
            let r = if let Some(MemoData(opponent_score, _, MemoType::Exact(_))) = memo {
                -*opponent_score * BIG_NUMBER
            } else {
                -EVAL_UPPER_BOUND * BIG_NUMBER
            };

            self.test_board.revert_move(&revertable);
            (*m).1 = r;
        }
        */

        if moves_start == moves_end_exclusive {
            return self.get_no_moves_eval(alpha, beta);
        }

        evaluation::add_captures_to_evals(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive);
        evaluation::add_mobility_to_evals(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive);
        self.moves_buf.sort_subset_by_eval(moves_start, moves_end_exclusive);

        for i in (moves_start..moves_end_exclusive).rev() {
            // TODO Review borrowing again
            // First, review Vec again, and [] being immutable ref
            // Then, is the problem: move itself obv immutable but chain cascades into self (too much), 
            // if self is immutable, that makes no sense as a recursive algo with potentially mutable buffers
            let m: *const MoveWithEval = &self.moves_buf.v()[i];

            let m_score = (*m).1;
            // TODO Extract this if trick as macrorules. Unit tests.
            let mut less_depth_amount = (-((remaining_depth > 3 && new_alpha_i != NEW_ALPHA_I_NEVER_SET) as i32)) 
                & min(-((m_score < 100) as i32) & ((100 - m_score) >> 5), 3);

            // TODO Statistics for research, number of retries
            loop {
                let r = self.negamax_try_move(
                    remaining_depth,
                    remaining_depth - (less_depth_amount as i8), 
                    alpha,
                    new_alpha_i != NEW_ALPHA_I_NEVER_SET,
                    beta,
                    m,
                    moves_end_exclusive
                );

                // TODO Review soundness. I believe this says if move ordering is perfect, then 
                // late move reduction call will never trigger any changes in the best move. 
                if let SingleMoveResult::NewAlpha(score) = r {
                    assert!(!self.terminated); // TODO Why?
                    if less_depth_amount <= 0 {
if remaining_depth >= self.iterative_deepening_depth {
    console_log!("--new alpha: {}", score);
}
                        alpha = score;
                        new_alpha_i = i as i32;
                        break;
                    } else {
                        less_depth_amount = 0;
                    }
                } else if let SingleMoveResult::BetaCutOff(score) = r {
                    if self.terminated {
                        // Don't lose the best move so far during termination, but don't pretend it's the real move at this depth (hence -1 to remaining depth)
                        if new_alpha_i == NEW_ALPHA_I_HASH_MOVE {
                            self.insert_memo(MemoData(alpha, remaining_depth - 1, MemoType::Exact(hash_move.unwrap().clone()), 0));
                        } else if new_alpha_i >= 0 {
                            self.insert_memo(MemoData(alpha, remaining_depth - 1, MemoType::Exact(self.moves_buf.v()[new_alpha_i as usize].clone()), 0));
                        }
                        return initial_alpha; // See (2)
                    } else {
                        if less_depth_amount <= 0 {
                            self.insert_memo(MemoData(score, remaining_depth, MemoType::GreaterThan((*m).clone()), 0));
                            return beta;
                        } else {
                            less_depth_amount = 0;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        assert!(!self.terminated); // TODO Review why?
        if new_alpha_i == NEW_ALPHA_I_HASH_MOVE {
            self.insert_memo(MemoData(alpha, remaining_depth, MemoType::Exact(hash_move.unwrap()), 0));
        } else if new_alpha_i >= 0 {
            self.insert_memo(MemoData(alpha, remaining_depth, MemoType::Exact(self.moves_buf.v()[new_alpha_i as usize].clone()), 0));
        } else {
            // Even when there is no alternative to losing via checkmate (never > alpha), need to give a best move. 
            self.insert_memo(MemoData(alpha, remaining_depth, MemoType::LessThan(self.moves_buf.v()[moves_start].clone()), 0));
        }
if remaining_depth >= self.iterative_deepening_depth {
console_log!("check alpha: {}", alpha);
}

        alpha
    }

    /// Unsafe purpose: allow reference to `m` while the move list holding it is being mutated, trusting proper management of move list subsets.
    unsafe fn negamax_try_move(
        &mut self,
        remaining_depth_no_lmr: i8,
        remaining_depth: i8,
        alpha: i32,
        do_null_window: bool,
        beta: i32,
        m: *const MoveWithEval,
        moves_start: usize
    ) -> SingleMoveResult {
        let mut revertable = RevertableMove::NoOp(0);
        let before_move_hash = self.test_board.get_hash();
        self.test_board.handle_move(&*m, &mut revertable);

        // TODO (Minor) Inline func helper "is_first_depth"
        if remaining_depth_no_lmr >= self.iterative_deepening_depth {
            let current_hash = self.test_board.get_hash();
            let unrawed: &Vec<u64> = &*self.real_board_positional_hashes;
            let repetition_count = unrawed.iter().filter(|&&h| h == current_hash).count();
            console_log!("? {} reps={}, real#={} remaining_depth_no_lmr={} it={} {} {} {}", self.test_board.stringify_move_for_js_logs(&*m), repetition_count, unrawed.len(), remaining_depth_no_lmr, self.iterative_deepening_depth, unrawed as *const Vec<u64> as usize, alpha, beta);
            if repetition_count >= 2 {
                console_log!("GREATER 2");
                self.test_board.revert_move(&revertable);
                console_log!("HASH THERE 1? {}", self.memo.contains_key(&before_move_hash));
                self.memo.remove(&before_move_hash);
                console_log!("HASH THERE 2? {} alpha={} beta={}", self.memo.contains_key(&before_move_hash), alpha, beta);
                if 0 >= beta {
                    return SingleMoveResult::BetaCutOff(beta);
                } else if 0 > alpha {
                    return SingleMoveResult::NewAlpha(0);
                } else {
                    return SingleMoveResult::NoEffect;
                }
            } else {
                self.moves_buf.write_index = moves_start;
                self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
                let moves_end_exclusive = self.moves_buf.write_index;

                let mut repetition = false;
                for i in (moves_start..moves_end_exclusive).rev() {
                    let m: *const MoveWithEval = &self.moves_buf.v()[i];

                    let mut revertable2 = RevertableMove::NoOp(0);
                    self.test_board.handle_move(&*m, &mut revertable2);
                    let current_hash = self.test_board.get_hash();
                    let repetition_count = unrawed.iter().filter(|&&h| h == current_hash).count();
                    if repetition_count >= 2 {
                        console_log!("?2 GREATER 2");
                        console_log!("HASH THERE 1?2 {}", self.memo.contains_key(&before_move_hash));
                        self.memo.remove(&before_move_hash);
                        console_log!("HASH THERE 2?2 {}", self.memo.contains_key(&before_move_hash));
                        self.test_board.revert_move(&revertable2);
                        repetition = true;
                        break;
                    } else {
                        self.test_board.revert_move(&revertable2);
                    }
                }

                if repetition {
                    self.test_board.revert_move(&revertable);
                    if 0 >= beta {
                        return SingleMoveResult::BetaCutOff(beta);
                    } else if 0 > alpha {
                        return SingleMoveResult::NewAlpha(0);
                    } else {
                        return SingleMoveResult::NoEffect;
                    }
                }
            }
        }

        // TODO IMMEDIATE If top level, print console log, comment minimal printing, 
        // check ref to pos hashes for repetition, if so, return as fail low, comment same as "move sucked",
        // so goes on to next move, comment this means (for speed) immediately, when AI at top level wants to 
        // recurse to search, it discovers the position is a repetition and avoids the move 

        let mut fast_found_score: i32 = 0;
        let mut fast_found = false;

        if do_null_window {

            // [PVS intuition]
            // In the ideal alpha-beta setup, the first move is the best move, set a new alpha and no new alpha is set again.
            // In this case, we do the same alpha-beta (no null window) for the first move, followed by as small a window as possible
            // for subsequent moves (faster), betting on fail-lows. (If we do null window on first move, then it'll frequently re-search.)

            fast_found_score = -self.negamax(remaining_depth - 1, -alpha - 1, -alpha, moves_start);
            if fast_found_score <= alpha {
                fast_found = true;
                self.fast_found_hits += 1;
            }
        } 

        let score = if fast_found {
            fast_found_score
        } else {
            -self.negamax(remaining_depth - 1, -beta, -alpha, moves_start)
        };

        self.test_board.revert_move(&revertable);
if remaining_depth_no_lmr >= self.iterative_deepening_depth {
    console_log!("ntrymove {} {} {}", score, fast_found, alpha);
}

        if score >= beta {
            SingleMoveResult::BetaCutOff(score)
        } else if score > alpha {
            let extra = score - alpha;
            if remaining_depth >= self.iterative_deepening_depth && extra < RANDOMIZATION_DIFF {
                console_log!("Randomizing, extra = {}", extra);
                if random() > 0.5 {
                    SingleMoveResult::NoEffect
                } else {
                    SingleMoveResult::NewAlpha(score)
                }
            } else {
                SingleMoveResult::NewAlpha(score)
            }
        } else {
            SingleMoveResult::NoEffect
        }
    }
}
