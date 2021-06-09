use std::collections::HashSet;
use std::cell::{RefCell};
use super::game::coords::*;
use super::game::move_list::*;
use super::game::board::*;
use super::game::castle_utils::*;
use super::game::entities::*;
use super::game::basic_move_test::*;

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves_for_board: MoveList,
    moves1: MoveList,
    moves2: MoveList
}

static MAX_EVAL: f32 = 9000.;

impl Ai {

    pub fn evaluate_player(board: &Board, neg_one_if_white_else_one: f32, seven_if_white_else_zero: f32, locs: &HashSet<Coord>) -> f32 {
        let mut value: f32 = 0.;
        for Coord(x, y) in locs.iter() {
            let fy = *y as f32;

            if let Square::Occupied(piece, _) = board.get_by_xy(*x, *y) {

                let unadvanced = (3.5 - fy).abs();

                let piece_value = match piece {
                    Piece::Queen => 9.,
                    Piece::Pawn => 1.,
                    Piece::Rook => 5.,
                    Piece::Bishop => 3.,
                    Piece::Knight => 3.,
                    _ => 0.
                };
                value += piece_value;
                value += match piece {
                    Piece::Pawn => {
                        seven_if_white_else_zero + neg_one_if_white_else_one * fy * 0.3
                    },
                    _ => {
                        0.3 * (3.5 - unadvanced)
                    }
                };
            }
        }
        value * neg_one_if_white_else_one as f32 * -1.
    }

    pub fn evaluate(board: &Board) -> f32 {
        Ai::evaluate_player(board, 1., 0., &board.get_player_state(Player::Black).piece_locs) +
            Ai::evaluate_player(board, -1., 7., &board.get_player_state(Player::White).piece_locs)
    }

    pub fn new() -> Ai {
        Ai {
            moves_buf: MoveList::new(1000),
            test_board: Board::new(),
            temp_moves_for_board: MoveList::new(50),
            moves1: MoveList::new(50),
            moves2: MoveList::new(50)
        }
    }

    pub fn make_move(&mut self, castle_utils: &CastleUtils, depth: u8, real_board: &mut Board) {

        self.test_board.clone_from(real_board);
        self.moves1.write_index = 0;
        self.moves2.write_index = 0;

        let m = self.test_board.get_player_with_turn().get_multiplier();
        let evaluation = m * self.alpha_beta(
            castle_utils,
            depth, 
            -MAX_EVAL,
            MAX_EVAL,
            0,
            false
        );
        println!("m1/m2 {} {}", self.moves1.write_index, self.moves2.write_index);

        self.moves2.print(0, self.moves2.write_index);

        if self.moves2.write_index <= 0 {
            println!("No moves, checkmate?");
        } else {
            let best_move = &self.moves2.get_v()[self.moves2.write_index - 1];
            println!("Best: {}", best_move);
            real_board.make_move(best_move);
        }
        println!("\n{}\n", real_board);
        println!("Eval = {}, moves len = {}", evaluation, self.moves_buf.get_v().len());
    }

    // TODO Try observing iterative deepening on 2 Q vs 1 K because large space
    /// We have ownership over all move list elements from `moves_start`
    fn alpha_beta(
        &mut self,
        castle_utils: &CastleUtils,
        remaining_depth: u8,
        mut alpha: f32,
        beta: f32,
        moves_start: usize,
        use_moves1_as_trace_result: bool
    ) -> f32 {

        let node_start_moves1_index = self.moves1.write_index;
        let node_start_moves2_index = self.moves2.write_index;

        let current_player = self.test_board.get_player_with_turn();
        let opponent = current_player.get_other_player();

        let mut is_best_in_moves1 = true;
        let mut best_move: Option<*const MoveSnapshot> = None;

        let mut initialized_a_move = false;

        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(castle_utils, &mut self.temp_moves_for_board, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        for i in moves_start..moves_end_exclusive {

            self.test_board.make_move(&self.moves_buf.get_v()[i]);
            let max_this: f32 = if remaining_depth > 0 {
                -self.alpha_beta(castle_utils, remaining_depth - 1, -beta, -alpha, moves_end_exclusive, !is_best_in_moves1)
            } else {
                // eg. Black is the opponent, we are white -> 1.0 multiplier -> prefer higher evaluations 
                -opponent.get_multiplier() * Ai::evaluate(&self.test_board)
            };
            self.test_board.undo_move(&self.moves_buf.get_v()[i]);

            // Remember: If moves are equal, then act as if candidate is inferior, to save computation
            if max_this >= beta {
                return beta;
            } else if max_this > alpha { // Must be >= to always set move if one exists FIXME ??????????????
                let m = &self.moves_buf.get_v()[i];

                alpha = max_this;
                best_move = Some(m);

                // Devalue forced mate based on # of moves to do it
                if alpha == MAX_EVAL {
                    let l = if is_best_in_moves1 {
                        self.moves2.write_index - node_start_moves2_index
                    } else {
                        self.moves1.write_index - node_start_moves1_index
                    };
                    alpha -= l as f32;
                }

                if is_best_in_moves1 {
                    self.moves1.write_index = node_start_moves1_index;
                } else {
                    self.moves2.write_index = node_start_moves2_index;
                }
                is_best_in_moves1 = !is_best_in_moves1;

                initialized_a_move = true;
            } else {
                if is_best_in_moves1 {
                    self.moves2.write_index = node_start_moves2_index;
                } else {
                    self.moves1.write_index = node_start_moves1_index;
                }
            }
        }

        if initialized_a_move {
            if is_best_in_moves1 {
                unsafe {
                    self.moves1.write(*best_move.unwrap());
                }
                self.moves2.write_index = node_start_moves2_index;
            } else {
                unsafe {
                    self.moves2.write(*best_move.unwrap());
                }
                self.moves1.write_index = node_start_moves1_index;
            }

            if use_moves1_as_trace_result != is_best_in_moves1 {
                if is_best_in_moves1 {
                    for i in node_start_moves1_index..self.moves1.write_index {
                        self.moves2.write(self.moves1.get_v()[i]);
                    }
                    self.moves1.write_index = node_start_moves1_index;
                } else {
                    for i in node_start_moves2_index..self.moves2.write_index {
                        self.moves1.write(self.moves2.get_v()[i]);
                    }
                    self.moves2.write_index = node_start_moves2_index;
                }
            }

            alpha
        } else {
            println!("... Potentially no moves for {:?}: \n\n{}\n", current_player, self.test_board);

            self.moves_buf.write_index = moves_start;
            BasicMoveTest::fill_player(current_player.get_other_player(), &self.test_board, true, &mut self.moves_buf);
            if BasicMoveTest::has_king_capture_move(&self.moves_buf, moves_start, self.moves_buf.write_index, current_player) {
                -MAX_EVAL
            } else {
                0. // Stalemate
            }
        }
    }
}
