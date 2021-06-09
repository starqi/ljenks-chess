use std::cell::{RefCell};
use super::game::coords::*;
use super::game::move_list::*;
use super::game::board::*;
use super::game::castle_utils::*;
use super::game::entities::*;
use super::game::basic_move_test::*;

#[derive(Default)]
struct BestMove {
    best_move: Option<MoveSnapshot>,
    move_list_index: usize
}

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves_for_board: MoveList,
    moves1: MoveList,
    moves2: MoveList
}

static MAX_EVAL: f32 = 9000.;
static MIN_EVAL: f32 = -MAX_EVAL;

impl Ai {

    pub fn evaluate_player(board: &Board, player: Player) -> f32 {
        let mut value: f32 = 0.;
        for Coord(x, y) in board.get_player_state(player).piece_locs.iter() {
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
                        let x = match player {
                            Player::White => 7. - fy,
                            Player::Black => fy
                        };
                        x * x * 0.075
                    },
                    _ => {
                        0.3 * (3.5 - unadvanced)
                    }
                };
            }
        }
        if player == Player::Black {
            value *= -1.;
        }
        value
    }

    pub fn evaluate(board: &Board) -> f32 {
        Ai::evaluate_player(board, Player::Black) + Ai::evaluate_player(board, Player::White)
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
        let best_move: RefCell<BestMove> = RefCell::new(Default::default());
        let evaluation = self.alpha_beta(
            castle_utils,
            depth, 
            MIN_EVAL,
            MAX_EVAL,
            0,
            false,
            Some(&best_move)
        );
        println!("m1/m2 {} {}", self.moves1.write_index, self.moves2.write_index);

        self.moves2.print(0, self.moves2.write_index);

        let best_move_inner = best_move.borrow();
        if best_move_inner.best_move.is_none() {
            println!("No moves, checkmate?");
        } else {
            println!("Best: {}", best_move_inner.best_move.unwrap());
            real_board.make_move(&self.moves_buf.get_v()[best_move_inner.move_list_index]);
        }
        println!("\n{}\n", real_board);
        println!("Eval = {}, moves len = {}", evaluation, self.moves_buf.get_v().len());
    }

    // TODO Try observing iterative deepening on 2 Q vs 1 K because large space
    // FIXME Need to output principle path or at least length to tie break quicker mates, so you don't infinitely chase after the longer one
    // FIXME Check detection if no moves to distinguish stalemate
    // TODO Both players are maximizing
    /// We have ownership over all move list elements from `moves_start`
    fn alpha_beta(
        &mut self,
        castle_utils: &CastleUtils,
        depth: u8,
        alpha: f32,
        beta: f32,
        moves_start: usize,
        use_moves1_as_trace_result: bool,
        best_move_result: Option<&RefCell<BestMove>>
    ) -> f32 {

        let node_start_moves1_index = self.moves1.write_index;
        let node_start_moves2_index = self.moves2.write_index;

        let current_player = self.test_board.get_player_with_turn();

        let mut is_best_in_moves1 = true;
        let mut best_move: Option<*const MoveSnapshot> = None;
        let mut best_min = MAX_EVAL;

        let mut initialized_a_move = false;

        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(castle_utils, &mut self.temp_moves_for_board, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        for i in moves_start..moves_end_exclusive {

            self.test_board.make_move(&self.moves_buf.get_v()[i]);

            let opponent_best_value: f32 = if depth > 0 {

                let new_alpha: f32;
                let new_beta: f32;

                if current_player == Player::White {
                    new_beta = beta;
                    new_alpha = if -best_min > alpha {
                        -best_min
                    } else {
                        alpha
                    }
                } else {
                    new_beta = if best_min < beta {
                        best_min
                    } else {
                        beta
                    };
                    new_alpha = alpha;
                }

                self.alpha_beta(castle_utils, depth - 1, new_alpha, new_beta, moves_end_exclusive, !is_best_in_moves1, None)
            } else {
                Ai::evaluate(&self.test_board)
            };

            // Turn maximizing player into minimization problem
            let mut value_to_minimize = if current_player == Player::White {
                opponent_best_value * -1.
            } else {
                opponent_best_value
            };

            if value_to_minimize == MIN_EVAL {
                let l = if is_best_in_moves1 {
                    self.moves2.write_index - node_start_moves2_index
                } else {
                    self.moves1.write_index - node_start_moves1_index
                };
                value_to_minimize += l as f32;
            }

            // Must be <= to always set move if one exists
            if value_to_minimize <= best_min {
                let m = &self.moves_buf.get_v()[i];

                best_min = value_to_minimize;
                best_move = Some(m);

                if is_best_in_moves1 {
                    self.moves1.write_index = node_start_moves1_index;
                } else {
                    self.moves2.write_index = node_start_moves2_index;
                }
                is_best_in_moves1 = !is_best_in_moves1;

                initialized_a_move = true;
                if let Some(a) = best_move_result {
                    let mut b = a.borrow_mut();
                    unsafe {
                        b.best_move = Some(*best_move.unwrap());
                    }
                    b.move_list_index = i;
                }
            } else {
                if is_best_in_moves1 {
                    self.moves2.write_index = node_start_moves2_index;
                } else {
                    self.moves1.write_index = node_start_moves1_index;
                }
            }

            self.test_board.undo_move(&self.moves_buf.get_v()[i]);

            if current_player == Player::White {
                let best_max = -best_min;
                if best_max >= beta {
                    break;
                }
            } else {
                if best_min <= alpha {
                    break;
                }
            }
        }

        if initialized_a_move {
            let eval = if current_player == Player::White { -best_min } else { best_min };

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

            eval
        } else {
            println!("... Potentially no moves for {:?}: \n\n{}\n", current_player, self.test_board);

            self.moves_buf.write_index = moves_start;
            BasicMoveTest::fill_player(current_player.get_other_player(), &self.test_board, true, &mut self.moves_buf);
            if BasicMoveTest::has_king_capture_move(&self.moves_buf, moves_start, self.moves_buf.write_index, current_player) {
                if current_player == Player::White { MIN_EVAL } else { MAX_EVAL }
            } else {
                0. 
            }
        }
    }
}
