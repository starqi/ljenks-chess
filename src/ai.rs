use std::cell::{RefCell};
use std::{thread, option_env};
use std::collections::{HashSet};
use rand::{ThreadRng, thread_rng, Rng};
use super::board::{Coord, Player, Board, MoveList, Piece, Square, xy_to_file_rank_safe};
use std::sync::{Mutex, Arc};
use log::{debug, info, warn, error};

#[derive(Default)]
struct BestMove {
    piece_loc: Coord,
    dest_loc: Coord,
    move_list_index: usize,
    written: bool
}

pub struct Ai {
    moves_buf: MoveList,
    test_board: Board,
    temp_moves_for_board: MoveList,
    moves1: MoveList,
    moves2: MoveList,
    rng: ThreadRng,
    pub counter: Arc<Mutex<u32>>
}

static MAX_EVAL: f32 = 9001.;
static MIN_EVAL: f32 = -MAX_EVAL;

impl Ai {

    pub fn evaluate_player(board: &Board, player: Player) -> f32 {
        let mut value: f32 = 0.;
        for (x, y) in board.get_player_state(player).piece_locs.iter() {
            let fy = *y as f32;

            if let Ok(Square::Occupied(piece, _)) = board.get_by_xy(*x, *y) {

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
            moves2: MoveList::new(50),
            rng: thread_rng(),
            counter: Arc::new(Mutex::new(0))
        }
    }

    pub fn make_move(&mut self, depth: u8, real_board: &mut Board) {
        *self.counter.lock().unwrap() = 0;

        self.test_board.import_from(real_board);
        self.moves1.write_index = 0;
        self.moves2.write_index = 0;
        let best_move: RefCell<BestMove> = RefCell::new(Default::default());
        let evaluation = self.alpha_beta(
            depth, 
            MIN_EVAL,
            MAX_EVAL,
            0,
            false,
            Some(&best_move)
        );
        println!("m1/m2 {} {}", self.moves1.write_index, self.moves2.write_index);

        for i in (0..self.moves2.write_index).rev() {
            let (src, dest, eval) = self.moves2.get_v()[i];
            let (src_f, src_r) = xy_to_file_rank_safe(src.0 as i32, src.1 as i32).unwrap();
            let (dest_f, dest_r) = xy_to_file_rank_safe(dest.0 as i32, dest.1 as i32).unwrap();
            println!("{}{} to {}{}, eval={}", src_f, src_r, dest_f, dest_r, eval);
        }
        println!("");

        let best_move_inner = best_move.borrow();
        if !best_move_inner.written {
            println!("No moves, checkmate?");
        } else {
            let (src, dest, _) = &self.moves_buf.get_v()[best_move_inner.move_list_index];
            let (src_file, src_rank) = xy_to_file_rank_safe(src.0 as i32, src.1 as i32).unwrap();
            let (dest_file, dest_rank) = xy_to_file_rank_safe(dest.0 as i32, dest.1 as i32).unwrap();

            println!(
                "{:?} moves {}{} to {}{}", real_board.get_player_with_turn(), src_file, src_rank, dest_file, dest_rank
            );
            real_board.make_move(&mut self.moves_buf, best_move_inner.move_list_index);
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
        depth: u8,
        alpha: f32,
        beta: f32,
        moves_start: usize,
        use_moves1_as_trace_result: bool,
        best_move: Option<&RefCell<BestMove>>
    ) -> f32 {

        let node_start_moves1_index = self.moves1.write_index;
        let node_start_moves2_index = self.moves2.write_index;

        let current_player = self.test_board.get_player_with_turn();

        let mut is_best_in_moves1 = true;
        let mut best_min = MAX_EVAL;
        let mut best_src: Coord = (0, 0);
        let mut best_dest: Coord = (0, 0);

        let mut TODO = false;

        self.moves_buf.write_index = moves_start;
        self.test_board.get_moves(&mut self.temp_moves_for_board, &mut self.moves_buf);
        let moves_end_exclusive = self.moves_buf.write_index;

        for i in moves_start..moves_end_exclusive {

            let revertable = self.test_board.get_revertable_move(&self.moves_buf, i);
            self.test_board.make_move(&mut self.moves_buf, i);

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

                self.alpha_beta(depth - 1, new_alpha, new_beta, moves_end_exclusive, !is_best_in_moves1, None)
            } else {
                let mut counter = self.counter.lock().unwrap();
                *counter += 1;

                Ai::evaluate(&self.test_board)
            };

            // Turn maximizing player into minimization problem
            let value_to_minimize = if current_player == Player::White {
                opponent_best_value * -1.
            } else {
                opponent_best_value
            };

            // Must be <= to always set move if one exists
            if value_to_minimize <= best_min {
                let m = &self.moves_buf.get_v()[i];

                best_min = value_to_minimize;
                best_src = m.0;
                best_dest = m.1;

                if is_best_in_moves1 {
                    self.moves1.write_index = node_start_moves1_index;
                } else {
                    self.moves2.write_index = node_start_moves2_index;
                }
                is_best_in_moves1 = !is_best_in_moves1;

                TODO = true;
                if let Some(a) = best_move {
                    let mut b = a.borrow_mut();
                    let m = &self.moves_buf.get_v()[i];
                    b.piece_loc = m.0;
                    b.dest_loc = m.1;
                    b.move_list_index = i;
                    b.written = true;
                }
            } else {
                if is_best_in_moves1 {
                    self.moves2.write_index = node_start_moves2_index;
                } else {
                    self.moves1.write_index = node_start_moves1_index;
                }
            }

            self.test_board.revert_move(&revertable);

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

        let eval = if current_player == Player::White { -best_min } else { best_min };
        if TODO {

            if is_best_in_moves1 {
                self.moves1.write(best_src, best_dest, eval);
                self.moves2.write_index = node_start_moves2_index;
            } else {
                self.moves2.write(best_src, best_dest, eval);
                self.moves1.write_index = node_start_moves1_index;
            }

            if use_moves1_as_trace_result != is_best_in_moves1 {
                if is_best_in_moves1 {
                    for i in node_start_moves1_index..self.moves1.write_index {
                        let (src, dest, eval) = self.moves1.get_v()[i];
                        self.moves2.write(src, dest, eval);
                    }
                    self.moves1.write_index = node_start_moves1_index;
                } else {
                    for i in node_start_moves2_index..self.moves2.write_index {
                        let (src, dest, eval) = self.moves2.get_v()[i];
                        self.moves1.write(src, dest, eval);
                    }
                    self.moves2.write_index = node_start_moves2_index;
                }
            }
        } else {
            println!("... Potentially no moves for {:?}: \n\n{}\n", current_player, self.test_board);
        }

        eval
    }
}
