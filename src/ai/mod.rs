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
    memo: HashMap<i128, EvaluationAndDepth>,
    memo_hits: usize,
    fast_found_hits: usize
}

struct EvaluationAndDepth(f32, u8);

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

    pub fn make_move(&mut self, mut depth: u8, real_board: &mut Board) {

        assert!(depth >= 1);
        depth -= 1;

        self.test_board.clone_from(real_board);

        self.moves_buf.write_index = 0;
        self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;
        if moves_end_exclusive == 0 {
            console_log!("No legal moves");
        } else {

            console_log!("\n{}\n", real_board);

            crate::console_log!("\nMove ordering DEBUG");
            evaluation::sort_moves_by_aggression(&self.test_board, &mut self.moves_buf, 0, moves_end_exclusive, &mut self.eval_temp_arr, &mut self.temp_moves);
            self.moves_buf.print(0, moves_end_exclusive);

            crate::console_log!("\nControl DEBUG");
            for temp_arr_x in 0..8 {
                let start = temp_arr_x * 8;
                console_log!("{:?}", &self.eval_temp_arr[start..start + 8]);
            }

            for d in 0..=depth {
                for i in (0..moves_end_exclusive).rev() {
                    self.test_board.make_move(&self.moves_buf.get_v()[i]);
                    let evaluation_as_maximizer = -self.negamax(
                        d,
                        -MAX_EVAL,
                        MAX_EVAL,
                        moves_end_exclusive
                    );
                    self.moves_buf.get_mutable_snapshot(i).1 = evaluation_as_maximizer;
                    self.test_board.undo_move(&self.moves_buf.get_v()[i]);
                }
                self.moves_buf.sort_subset_by_eval(0, moves_end_exclusive);

                let leading_move = &self.moves_buf.get_v()[moves_end_exclusive - 1];
                console_log!("Depth {} - Best (real eval): {}", d + 1, leading_move);
            }
            self.moves_buf.print(0, moves_end_exclusive);
            let best_move = &self.moves_buf.get_v()[moves_end_exclusive - 1];
            console_log!("Making move: {}", best_move);
            real_board.make_move(best_move);
            console_log!("\n{}\n", real_board);
            console_log!("{}", evaluation::evaluate(real_board, &mut self.eval_temp_arr));

            console_log!("Memo hits - {}, size - {}, fast found - {}", self.memo_hits, self.memo.len(), self.fast_found_hits);
            self.memo_hits = 0;
            self.fast_found_hits = 0;
            self.memo.clear();
        }
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

        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(&mut self.temp_moves, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        evaluation::sort_moves_by_aggression(&self.test_board, &mut self.moves_buf, moves_start, moves_end_exclusive, &mut self.eval_temp_arr, &mut self.temp_moves);

        let mut one_between_node_found = false;
        for i in (moves_start..moves_end_exclusive).rev() {

            self.test_board.make_move(&self.moves_buf.get_v()[i]);
            let as_num = self.test_board.as_number();

            let mut memo: Option<&EvaluationAndDepth> = self.memo.get(&as_num);
            if let Some(EvaluationAndDepth(_, depth)) = memo {
                if *depth < remaining_depth - 1 {
                    memo = None;
                }
            }

            let max_this: f32 = if let Some(EvaluationAndDepth(saved_max_this, _)) = memo {
                self.memo_hits += 1;
                *saved_max_this
            } else {

                let mut fast_found_max_this = 0.0f32;
                let mut fast_found = false;

                if one_between_node_found {
                    fast_found_max_this = -self.negamax(remaining_depth - 1, -alpha - 1., -alpha, moves_end_exclusive);
                    if fast_found_max_this <= alpha {
                        fast_found = true;
                        self.fast_found_hits += 1;
                    }
                } 

                if fast_found {
                    fast_found_max_this
                } else {
                    let a = -beta;
                    let b = -alpha;
                    let eval = self.negamax(remaining_depth - 1, a, b, moves_end_exclusive);
                    let _max_this = -eval;
                    if eval > a && eval < b {
                        // Only save exact evals
                        self.memo.insert(as_num, EvaluationAndDepth(_max_this, remaining_depth - 1));
                    }
                    _max_this
                }
            };

            self.test_board.undo_move(&self.moves_buf.get_v()[i]);

            if max_this >= beta {
                return beta;
            } else if max_this > alpha {
                alpha = max_this;
                one_between_node_found = true;
            }
        }

        alpha
    }
}
