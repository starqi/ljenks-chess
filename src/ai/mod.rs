mod evaluation;

use std::collections::HashMap;
use super::game::move_list::*;
use super::game::board::*;
use super::game::castle_utils::*;
use crate::{console_log};

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves_for_board: MoveList,
    memo: HashMap<i128, EvaluationAndDepth>,
    memo_hits: usize
}

struct EvaluationAndDepth(f32, u8);

static MAX_EVAL: f32 = 9000.;

impl Ai {

    pub fn new() -> Ai {
        console_log!("AI init");
        Ai {
            moves_buf: MoveList::new(1000),
            test_board: Board::new(),
            temp_moves_for_board: MoveList::new(50),
            memo: HashMap::new(),
            memo_hits: 0
        }
    }

    pub fn make_move(&mut self, castle_utils: &CastleUtils, depth: u8, real_board: &mut Board) {

        assert!(depth >= 1);

        self.test_board.clone_from(real_board);
        let m = self.test_board.get_player_with_turn().get_multiplier();

        self.moves_buf.write_index = 0;
        self.test_board.get_moves(castle_utils, &mut self.temp_moves_for_board, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;
        if moves_end_exclusive == 0 {
            console_log!("No legal moves");
        } else {
            let real_depth = depth - 1;
            for d in 0..real_depth {
                for i in (0..moves_end_exclusive).rev() {
                    self.test_board.make_move(&self.moves_buf.get_v()[i]);
                    let evaluation_as_maximizer = -self.negamax(
                        castle_utils,
                        d,
                        -MAX_EVAL,
                        MAX_EVAL,
                        moves_end_exclusive
                    );
                    self.moves_buf.get_mutable_snapshot(i).1 = evaluation_as_maximizer;
                    self.test_board.undo_move(&self.moves_buf.get_v()[i]);
                }
                self.moves_buf.sort_subset(0, moves_end_exclusive);
                let leading_move = &self.moves_buf.get_v()[moves_end_exclusive - 1];
                console_log!("Leading move: {}", leading_move);
            }
            self.moves_buf.print(0, moves_end_exclusive);
            let best_move = &self.moves_buf.get_v()[moves_end_exclusive - 1];
            console_log!("Making move: {}", best_move);
            real_board.make_move(best_move);
            console_log!("\n{}\n", real_board);
            console_log!("{}", evaluation::evaluate(real_board));
            console_log!("{}", real_board.as_number());
            console_log!("Memo hits - {}, size - {}", self.memo_hits, self.memo.len());
            self.memo.clear();
        }
    }

    /// Will assume ownership over all move list elements from `moves_start`
    /// Only calculates score
    fn negamax(
        &mut self,
        castle_utils: &CastleUtils,
        remaining_depth: u8,
        mut alpha: f32,
        beta: f32,
        moves_start: usize
    ) -> f32 {

        if remaining_depth <= 0 {
            let eval = evaluation::evaluate(&self.test_board);
            // FIXME What is official strategy?
            return self.test_board.get_player_with_turn().get_multiplier() * eval;
        }

        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(castle_utils, &mut self.temp_moves_for_board, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        for i in moves_start..moves_end_exclusive {

            self.test_board.make_move(&self.moves_buf.get_v()[i]);
            let as_num = self.test_board.as_number();

            let mut memo: Option<&EvaluationAndDepth> = self.memo.get(&as_num);
            if let Some(EvaluationAndDepth(_, depth)) = memo {
                if *depth < remaining_depth - 1 {
                    memo = None;
                }
            }

            let max_this: f32 = if let Some(EvaluationAndDepth(saved_eval, _)) = memo {
                self.memo_hits += 1;
                *saved_eval
            } else {
                let r = -self.negamax(castle_utils, remaining_depth - 1, -beta, -alpha, moves_end_exclusive);
                self.memo.insert(as_num, EvaluationAndDepth(r, remaining_depth - 1));
                r
            };

            self.test_board.undo_move(&self.moves_buf.get_v()[i]);

            if max_this >= beta {
                return beta;
            } else if max_this > alpha {
                alpha = max_this;
            }
        }

        alpha
    }
}