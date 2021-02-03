use std::cell::{RefCell};
use std::{thread, option_env};
use std::collections::{HashSet};
use rand::{ThreadRng, thread_rng, Rng};
use super::board::{Coord, Player, Board, MoveList, Piece, Square, CheckThreatTempBuffers, xy_to_file_rank_safe};
use std::sync::{Mutex, Arc};
use log::{debug, info, warn, error};

#[derive(Default)]
struct BestMove {
    piece_loc: Coord,
    move_list_index: usize,
    written: bool
}

pub struct Ai {
    temp_board: Board,
    check_bufs: CheckThreatTempBuffers,
    rng: ThreadRng,
    pub counter: Arc<Mutex<u32>>
}

static MAX_EVAL: f32 = 9001.;
static MIN_EVAL: f32 = -MAX_EVAL;

impl Ai {

    pub fn evaluate_player(board: &Board, rng: &mut ThreadRng, player: Player) -> f32 {
        let mut value: f32 = 0.;
        for (x, y) in board.get_player_state(player).piece_locs.iter() {
            if let Ok(Square::Occupied(piece, _)) = board.get_by_xy(*x, *y) {

                let unadvanced = (3.5 - (*y as f32)).abs();

                let piece_value = match piece {
                    Piece::Queen => 9.,
                    Piece::Pawn => 1.,
                    Piece::Rook => 5.,
                    Piece::Bishop => 3.,
                    Piece::Knight => 3.,
                    _ => 0.
                };

                value += piece_value + 0.1 * (3.5 - unadvanced);
            }
        }
        //value = ((value as f32) * rng.gen_range(0.8, 1.2)) as i32;
        if player == Player::Black {
            value *= -1.;
        }
        value
    }

    pub fn evaluate(board: &Board, rng: &mut ThreadRng) -> f32 {
        Ai::evaluate_player(board, rng, Player::Black) + Ai::evaluate_player(board, rng, Player::White)
    }

    pub fn new() -> Ai {
        Ai {
            temp_board: Board::new(),
            check_bufs: CheckThreatTempBuffers::new(),
            rng: thread_rng(),
            counter: Arc::new(Mutex::new(0))
        }
    }

    pub fn make_move(&mut self, depth: u8, real_board: &mut Board) {
        *self.counter.lock().unwrap() = 0;

        let mut move_list_buf_for_children = MoveList::new();
        let mut piece_locs_buf_for_children: HashSet<Coord> = HashSet::new();

        self.temp_board.import_from(real_board);
        let best_move: RefCell<BestMove> = RefCell::new(Default::default());
        let evaluation = self.alpha_beta(
            depth, MIN_EVAL, MAX_EVAL,
            &mut move_list_buf_for_children,
            &mut piece_locs_buf_for_children,
            Some(&best_move)
        );

        let best_move_inner = best_move.borrow();
        if !best_move_inner.written {
            println!("No moves, checkmate?");
        } else {
            let (file, rank) = xy_to_file_rank_safe(best_move_inner.piece_loc.0 as i32, best_move_inner.piece_loc.1 as i32).unwrap();
            real_board.get_moves(file, rank, &mut self.check_bufs, &mut move_list_buf_for_children).unwrap();

            let (dest_x, dest_y) = move_list_buf_for_children.get_moves().get(best_move_inner.move_list_index).unwrap();
            let (dest_file, dest_rank) = xy_to_file_rank_safe(*dest_x as i32, *dest_y as i32).unwrap();

            println!(
                "{:?} moves {}{} to {}{}",
                real_board.get_player_with_turn(),
                file, rank,
                dest_file, dest_rank
            );

            real_board.make_move(&mut move_list_buf_for_children, best_move_inner.move_list_index).unwrap();
        }
        println!("\n{}\n", real_board);
        println!("Eval = {}", evaluation);
    }

    fn alpha_beta(
        &mut self,
        depth: u8, alpha: f32, beta: f32,
        move_list_buf_from_above: &mut MoveList,
        piece_locs_buf_from_above: &mut HashSet<Coord>,
        best_move: Option<&RefCell<BestMove>>
    ) -> f32 {

        debug!("<{}", depth);

        let mut move_list_buf_for_children = MoveList::new();
        let mut piece_locs_buf_for_children: HashSet<Coord> = HashSet::new();

        let current_player = self.temp_board.get_player_with_turn();
        let current_player_state = self.temp_board.get_player_state(current_player);

        let mut best_min = MAX_EVAL;
        let mut TODO = false;

        piece_locs_buf_from_above.clone_from(&current_player_state.piece_locs);
        for (p_x, p_y) in piece_locs_buf_from_above.iter() {

            let (file, rank) = xy_to_file_rank_safe(*p_x as i32, *p_y as i32).unwrap();

            self.temp_board.get_moves(file, rank, &mut self.check_bufs, move_list_buf_from_above).unwrap();
            let moves_v = move_list_buf_from_above.get_moves();
            let moves_v_len = moves_v.len();

            for i in 0..moves_v_len {

                debug!("{} {}{} {}", depth, file, rank, i);

                let revertable = self.temp_board.get_revertable_move(move_list_buf_from_above, i).unwrap();
                self.temp_board.make_move(move_list_buf_from_above, i).unwrap();

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

                    self.alpha_beta(
                        depth - 1, new_alpha, new_beta,
                        &mut move_list_buf_for_children,
                        &mut piece_locs_buf_for_children,
                        None
                    )
                } else {
                    let mut counter = self.counter.lock().unwrap();
                    *counter += 1;

                    Ai::evaluate(&self.temp_board, &mut self.rng)
                };

                // Turn maximizing player into minimization problem
                let value_to_minimize = if current_player == Player::White {
                    opponent_best_value * -1.
                } else {
                    opponent_best_value
                };

                // Must be <= to always set move if one exists
                if value_to_minimize <= best_min {
                    best_min = value_to_minimize;
                    TODO = true;
                    if let Some(a) = best_move {
                        let mut b = a.borrow_mut();
                        b.piece_loc = (*p_x, *p_y);
                        b.move_list_index = i;
                        b.written = true;
                    }
                }

                self.temp_board.revert_move(&revertable).unwrap();

                if current_player == Player::White {
                    let best_max = -best_min;
                    if best_max > beta {
                        info!("falsified black {} vs {} {}/{}", best_max, beta, i, moves_v_len - 1);
                        return best_max;
                    }
                } else {
                    if best_min < alpha {
                        info!("falsified white {} vs {} {}/{}", best_min, alpha, i, moves_v_len - 1);
                        return best_min;
                    }
                }

            }
        }

        if !TODO {
            println!("... Potentially no moves for {:?}: \n\n{}\n", current_player, self.temp_board);
        }

        if current_player == Player::White { -best_min } else { best_min }
    }
}
