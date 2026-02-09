pub mod evaluation;
pub mod move_buckets;

pub use move_buckets::*;

use std::collections::HashMap;
use super::game::entities::*;
use super::game::move_test::*;
use super::game::move_list::*;
use super::game::board::*;
use crate::platform::{random, now};
use crate::branchless_mask;
use crate::{console_log};
use super::game::board::slow_stringify_move_standard;

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves: MoveList,
    af_boards: AttackFromBoards, // Deprecated
    memo: HashMap<u64, MemoData>,
    useful_memo_hits: usize,
    hash_move_memo_hits: usize,
    fast_found_hits: usize,
    node_counter: u64,
    start_ms: u128,
    // TODO IMMEDIATE Configurable config
    ms_till_terminate: u128, // Configuration
    terminated: bool,
    min_depth: i8, // Configuration
    iterative_deepening_depth: i8,
    real_board_positional_hashes: *const Vec<u64>,
    half_moves_without_pawn_move: *const usize,
    move_buckets: MoveBuckets,
}

// TODO (Minor) Rename this, NoEffect -> Fail low
/// BetaCutOff i32 will contain the precise score, it's not the upper bound.
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

/// Comprehensive info about the best move in a position.
pub struct BestMoveInfo {
    /// Helps with 50 move rule
    pub is_pawn: bool,
    pub remaining_depth: i8,
    pub score: i32,
    /// Standard chess notation
    pub notation: String,
    pub m: MoveWithEval,
}

impl Ai {

    /// Must be called, second "new". Had issues with wasm tracking Rust lifetimes, and couldn't do it in constructor...
    pub fn late_inject(&mut self, real_board_positional_hashes: &Vec<u64>, moves_without_pawn_move: &usize) {
        self.real_board_positional_hashes = real_board_positional_hashes;
        self.half_moves_without_pawn_move = moves_without_pawn_move;
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
            ms_till_terminate: 2000,
            terminated: false,
            min_depth: 7,
            iterative_deepening_depth: 1,
            real_board_positional_hashes: 0 as *const Vec<u64>,
            half_moves_without_pawn_move: 0 as *const usize,
            move_buckets: MoveBuckets::new(),
        }
    }

    /// Shared search func.
    /// Internal vars will mutate, memo will mutate (which contains best move), but nothing directly returned here.
    /// Interface is odd but this is performance code, and it's also private.
    /// (If this is called many times, eventually it will be too late to get the best move from memo 
    /// since it is outdated and removed.)
    fn run_search(&mut self, source_board: &Board) {
        self.test_board.clone_from(source_board);

        self.start_ms = now();
        self.terminated = false;
        self.node_counter = 0;
        self.useful_memo_hits = 0;
        self.hash_move_memo_hits = 0;
        self.fast_found_hits = 0;

        for d in (self.min_depth..=99).step_by(2) {
            console_log!("\nBegin depth {}", d);
            self.iterative_deepening_depth = d;
            unsafe {
                self.negamax(d, -MAX_EVAL, MAX_EVAL, 0);
            }

            let leading_move = self.get_leading_move(&self.test_board);
            if let Some((m, depth, _)) = leading_move {
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
    }

    fn log_search_stats(&self) {
        console_log!(
            "Useful memo hits - {}, hash move memo hits - {}, size - {}, fast found - {}, time - {}",
            self.useful_memo_hits,
            self.hash_move_memo_hits,
            self.memo.len(),
            self.fast_found_hits,
            now() - self.start_ms
        );
        console_log!("Nodes - {}, NPS - {}", self.node_counter, (self.node_counter as f64 / ((now() - self.start_ms) as f64 / 1000.)).round());
    }

    fn age_memo(&mut self) {

        console_log!("Memo aging, before size = {}", self.memo.len());
        for (_, MemoData(_, _, _, age)) in self.memo.iter_mut() {
            *age += 1;
        }
        self.memo.retain(|_, MemoData(_, _, _, age)| *age <= 5);
        console_log!("Memo aging, after size = {}", self.memo.len());
    }

    /// Pulls out the leading move from the memo given a board state.
    /// The memo needs to be generated first from search. 
    /// Ugly interface, private.
    fn get_leading_move(&self, board: &Board) -> Option<(&MoveWithEval, i8, i32)> {
        match self.memo.get(&board.get_hash()) {
            Some(MemoData(score, remaining_depth, MemoType::GreaterThan(best_move) | MemoType::Exact(best_move) | MemoType::LessThan(best_move), _)) => {
                Some((best_move, *remaining_depth, *score * board.get_player_with_turn().multiplier()))
            },
            _ => {
                None
            }
        }
    }

    /// Takes the leading move from the stateful memo, keyed by the hash of input board, and applies to the input board. 
    /// The memo needs to be generated first from search. 
    /// This process produces standard chess notation so a full `BestMoveInfo` is returned.
    /// Ugly interface, private.
    fn apply_leading_move(&self, real_board: &mut Board) -> Option<BestMoveInfo> {
        let leading_move_copy = if let Some((best_move, remaining_depth, score)) = self.get_leading_move(&real_board) {
            Some((best_move.clone(), remaining_depth, score))
        } else {
            None
        };
        if let Some((best_move, remaining_depth, score)) = leading_move_copy {
            let before_info = BeforeMoveInfoForStringify::slow_new(real_board, &best_move);
            let original_player = real_board.get_player_with_turn();
            let is_pawn = matches!(real_board.get_moved_piece(&best_move), Some(Piece::Pawn));

            real_board.handle_move_no_revert(&best_move);

            let is_check = real_board.is_checking(original_player);
            let is_checkmate = is_check && real_board.has_no_legal_moves();
            let after_info = AfterMoveInfoForStringify { is_check, is_checkmate };

            Some(BestMoveInfo {
                is_pawn,
                remaining_depth,
                score,
                notation: slow_stringify_move_standard(&best_move, &before_info, &after_info),
                m: best_move.clone()
            })
        } else {
            None
        }
    }

    pub fn make_move(&mut self, real_board: &mut Board) -> Option<BestMoveInfo> {
        self.run_search(real_board);
        let leading_move_ext = self.apply_leading_move(real_board);
        if let Some(ref x) = leading_move_ext {
            console_log!("Making move: {} (depth = {})", self.test_board.stringify_move_for_js_logs(&x.m), x.remaining_depth);
        } else {
            console_log!("No move");
        }
        self.log_search_stats();
        self.age_memo();
        leading_move_ext
    }

    pub fn evaluate(&mut self, board: &Board) -> Option<BestMoveInfo> {
        self.run_search(board);
        self.log_search_stats();
        let mut board2 = board.clone();
        self.apply_leading_move(&mut board2)
    }

    fn assert_king_pos(&self, player: Player) {
        if let Square::Occupied(Piece::King, _) = self.test_board.get_by_index(self.test_board.get_player_state(player).king_location._lsb_to_index()) {
        } else {
            panic!("Wrong king square detected for {:?}", player);
        }
    }

    // In high performance code, there is no formal concept of checkmate/stalemate/50 moves,
    // but simply whether there exist any moves at all, and if in check then it's checkmate.
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
    /// [Termination]
    /// Search should use a fake beta cutoff to escape recursively, 
    /// then check if `self.terminated` to see if it's a real beta cutoff.
    fn increment_node_check_termination(&mut self) -> bool {
        self.node_counter += 1;
        self.terminated = self.terminated || (
            self.node_counter % 50000 == 0 &&
            self.iterative_deepening_depth > self.min_depth &&
            now() - self.start_ms > self.ms_till_terminate
        );
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

        if self.increment_node_check_termination() { return initial_alpha; } // See [Termination]

        let mut alpha = initial_alpha;

        let score = if let (_, Some(adjusted_score)) = self.find_memo_score(0, alpha, beta) {
            self.useful_memo_hits += 1;
            return adjusted_score; // No score multiplier necessary
        } else {
            score_multiplier * evaluation::evaluate(&self.test_board)
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

        self.move_buckets.group_moves_qsearch(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive);
        let adjusted_end_exclusive = self.move_buckets.reorder_and_assign_scores_qsearch(&mut self.moves_buf, moves_start);

        for i in (moves_start..adjusted_end_exclusive).rev() {
            let m: *const MoveWithEval = &self.moves_buf.v()[i];
            let mut revertable = RevertableMove::NoOp(0);
            self.test_board.handle_move(&*m, &mut revertable);

            let r = -self.qsearch(
                remaining_depth_opt - 1, 
                -beta,
                -alpha,
                moves_end_exclusive // Not adjusted_end_exclusive
            );

            self.test_board.revert_move(&revertable);

            if r >= beta { 
                if self.terminated { return initial_alpha; } // See [Termination]
                return beta; 
            }
            if r > alpha { alpha = score; }
        }

        alpha
    }

    /// Will assume ownership over all move list elements from `moves_start`.
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
        if self.increment_node_check_termination() { return initial_alpha; } // See [Termination]

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

            // [Lame shallow fast draw detection scenario: when normal memo score is replaced with 0 (draw by repetition).]
            //
            // Normal scenario without this is returning the clamped score from memo.
            //
            // Now detect the memoized best move being actually a draw-by-repetition right before the draw is about to happen.
            // Detect the current player causing a draw OR current player allowing opponent to force draw (is_handled_move_shallow_draw).
            // Such a move will suddenly replace normal eval to score 0, "rewriting history".
            // Do this by replacing the memo, setting age such that it disappears immediately and doesn't interfere with future searches,
            // but lives long enough to not erase a move and end up with no moves.
            // For the sake of hash move, this is no longer a hash move, we can't say that the old non-0 non-draw eval is the first best move to consider.
            if remaining_depth >= self.iterative_deepening_depth {
                let mut revertable = RevertableMove::NoOp(0);
                let before_move_hash = self.test_board.get_hash();
                self.test_board.handle_move(&m, &mut revertable);
                // After this scope is done, other code can feel free to overwrite from index `moves_start`.
                let is_draw = self.is_handled_move_shallow_draw(moves_start);
                self.test_board.revert_move(&revertable);
                if is_draw {
                    self.replace_memo_for_draw(before_move_hash);
                } else {
                    console_log!("Hash move {}", clamped_score);
                    return clamped_score; 
                }
            } else {
                return clamped_score;
            }
        } else if let Some((m, _, false)) = memo_move_suggestion {

            if self.my_debug_enabled(remaining_depth) {
                console_log!("Starting search with memo move");
            }

            // Reminder: No null window, because this is our best move candidate, hence it is not expected to fail low
            match self.negamax_try_move(remaining_depth, remaining_depth, alpha, false, beta, &m, moves_start) {
                SingleMoveResult::BetaCutOff(score) => {
                    self.hash_move_memo_hits += 1;
                    if self.terminated {
                        return initial_alpha; // See [Termination]
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
            }
            assert!(!self.terminated);
        }

        // Generate moves and order
        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        if moves_start == moves_end_exclusive {
            return self.get_no_moves_eval(alpha, beta);
        }

        self.move_buckets.group_moves_normal(&self.test_board, &self.moves_buf, moves_start, moves_end_exclusive);
        let adjusted_end_exclusive = self.move_buckets.reorder_and_assign_scores(&mut self.moves_buf, moves_start);

        let hash_move2: MoveDescription = if let Some(inner_m) = &hash_move { inner_m.0.clone() } else { MoveDescription::SkipMove };

        for i in moves_start..adjusted_end_exclusive {
            // Rust philosophy reminder: unsafe pointer bypass because borrowing `m` chain borrows its owner, self, 
            // and `negamax_try_move` might modify self and `moves_buf` inside self, 
            // thus changing `m` while it's still being borrowed as immutable, "value changing underneath". 
            // But the move list start/end indices prevent this.
            let m: *const MoveWithEval = &self.moves_buf.v()[i];
            if (*m).0 == hash_move2 { continue; } // Skip hash move
            let m_score = (*m).1;

            // [LMR]
            // All normal depth except identify the obviously very quiet moves: 
            // neither attack nor capture and usually passive (e.g. backwards moves).
            // Our simple move ordering logic not good enough to aggressively prune.
            // Check real game logs for a feel of what is being pruned. 
            let less_depth_amount = branchless_mask!(remaining_depth > 1 && m_score >= NORMAL_SEARCH_POOR_MOVE_MIN_SCORE, 1);

            let r = self.negamax_try_move(
                remaining_depth,
                remaining_depth - (less_depth_amount as i8), 
                alpha,
                new_alpha_i != NEW_ALPHA_I_NEVER_SET,
                beta,
                m,
                moves_end_exclusive // Not adjusted_end_exclusive
            );

            if let SingleMoveResult::NewAlpha(score) = r {
                assert!(!self.terminated);
                alpha = score;
                new_alpha_i = i as i32;
            } else if let SingleMoveResult::BetaCutOff(score) = r {
                if self.terminated {
                    if new_alpha_i >= 0 {
                        self.insert_memo(MemoData(alpha, -1, MemoType::Exact(self.moves_buf.v()[new_alpha_i as usize].clone()), 0));
                    }
                    return initial_alpha; // See [Termination]
                } else {
                    self.insert_memo(MemoData(score, remaining_depth, MemoType::GreaterThan((*m).clone()), 0));
                    return beta;
                }
            }
        }

        assert!(!self.terminated);
        if new_alpha_i == NEW_ALPHA_I_HASH_MOVE {
            self.insert_memo(MemoData(alpha, remaining_depth, MemoType::Exact(hash_move.unwrap()), 0));
        } else if new_alpha_i >= 0 {
            self.insert_memo(MemoData(alpha, remaining_depth, MemoType::Exact(self.moves_buf.v()[new_alpha_i as usize].clone()), 0));
        } else {
            // Even when there is no alternative to losing via checkmate (never > alpha), need to give a best move. 
            self.insert_memo(MemoData(alpha, remaining_depth, MemoType::LessThan(self.moves_buf.v()[moves_start].clone()), 0));
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
        if self.my_debug_enabled(remaining_depth_no_lmr) {
            console_log!("negamax_try_move {}, depth = {}/{}",
                self.test_board.stringify_move_for_js_logs(&*m),
                remaining_depth,
                remaining_depth_no_lmr);
        }

        let mut revertable = RevertableMove::NoOp(0);
        let before_move_hash = self.test_board.get_hash();
        self.test_board.handle_move(&*m, &mut revertable);

        // [Lame shallow fast draw detection scenario: during negamax, part 2]
        // See other section with same name first. 
        // If draw is detected, then no need to recurse, follow same alpha-beta ideas but with score = 0.
        // Pretend we searched and got score = 0, and rewrite memo ("rewrite history") also with 0. Caller can't tell.
        // Remember to revert the move.

        // TODO (Minor) Inline func helper "is_first_depth"
        if remaining_depth_no_lmr >= self.iterative_deepening_depth {
            let is_draw = self.is_handled_move_shallow_draw(moves_start);
            if is_draw {
                console_log!("negamax_try_move.is_draw");
                self.replace_memo_for_draw(before_move_hash);
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

        let mut fast_found_score: i32 = 0;
        let mut fast_found = false;

        if do_null_window {
            // PVS intuition: minmize alpha-beta window for speed, hoping first move is best always
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

        if self.my_debug_enabled(remaining_depth_no_lmr) {
            if !self.terminated { // Don't print fake score when terminating
                console_log!("negamax_try_move.score alpha={}, beta={}, score={}, fast_found={}", alpha, beta, score, fast_found);
            }
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

    fn replace_memo_for_draw(&mut self, before_move_hash: u64) {
        if let Some(entry) = self.memo.get_mut(&before_move_hash) {
            entry.0 = 0; // Score
            entry.3 = 9999; // Age, pick a big number that it always expires immediately
        }
    }

    // See [Lame shallow fast draw detection scenario...] comments.
    unsafe fn is_handled_move_shallow_draw(&mut self, moves_start: usize) -> bool {
        let post_move_hash = self.test_board.get_hash();
        let real_board_positional_hashes: &Vec<u64> = &*self.real_board_positional_hashes;
        let repetition_count_wo_move = real_board_positional_hashes.iter().filter(|&&h| h == post_move_hash).count();

        if repetition_count_wo_move >= 2 {
            return true;
        } else {
            // Attempt #2 for opponent causing a draw
            self.moves_buf.write_index = moves_start;
            self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
            let moves_end_exclusive = self.moves_buf.write_index;
            for i in (moves_start..moves_end_exclusive).rev() {
                let m: *const MoveWithEval = &self.moves_buf.v()[i];

                let mut revertable2 = RevertableMove::NoOp(0);
                self.test_board.handle_move(&*m, &mut revertable2);
                let post_move_hash2 = self.test_board.get_hash();
                let repetition_count_wo_move2 = real_board_positional_hashes.iter().filter(|&&h| h == post_move_hash2).count();
                self.test_board.revert_move(&revertable2);
                if repetition_count_wo_move2 >= 2 {
                    return true;
                }
            }
            return false;
        }
    }

    #[cfg(feature = "my_debug")]
    #[inline]
    fn my_debug_enabled(&self, remaining_depth_no_lmr: i8) -> bool {
        remaining_depth_no_lmr >= self.iterative_deepening_depth
    }

    #[cfg(not(feature = "my_debug"))]
    #[inline]
    fn my_debug_enabled(&self, remaining_depth_no_lmr: i8) -> bool {
        false
    }
}
