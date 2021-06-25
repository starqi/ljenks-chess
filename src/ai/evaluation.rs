use super::super::game::entities::*;
use super::super::game::coords::*;
use super::super::game::board::*;
use super::super::game::move_list::*;
use super::super::game::move_test::*;
use super::super::game::push_moves_handler::*;

/// Must be bigger than all piece values
const NO_CONTROL_VAL: f32 = 99.;

static PIECE_VALUES: [i8; 6] = [
    1, 5, 3, 3, 9, 10
];

static SOME_VALUES_1: [(f32, f32); 2] = [(7., -1.), (0., 1.)];

struct SquareControlHandler<'a> {
    temp_arr: &'a mut [f32; 64]
}

pub fn clear_square_control(a: &mut [f32; 64]) {
    a.fill(NO_CONTROL_VAL);
}

#[inline]
pub fn get_square_worth_white(x: usize, y: usize) -> f32 {
    if y <= 1 { 1.0 }
    else if y >= 6 { 0.1 }
    else { 1.5 * (3.5 - (3.5 - x as f32).abs() + 0.4) / 4. }
}

#[inline]
pub fn get_square_worth_black(x: usize, y: usize) -> f32 {
    if y >= 6 { 1.0 }
    else if y <= 1 { 0.1 }
    else { 1.5 * (3.5 - (3.5 - x as f32).abs() + 0.4) / 4. }
}

/// Currently, returns at 4 for a center square, 3 for opponent side square
pub fn get_white_square_control(a: &mut [f32; 64]) -> f32 {
    let mut eval = 0.;
    for y in 0..8 {
        for x in 0..8 {
            let v = a[y * 8 + x];
            if v != NO_CONTROL_VAL && v.round() == v {
                if v < 0. {
                    eval -= get_square_worth_black(x, y);
                } else if v > 0. {
                    eval += get_square_worth_white(x, y);
                }
            }
        }
    }
    eval
}

/// Precondition: `temp_arr` is cleared to a number > the highest piece value before move tests
impl <'a> MoveTestHandler for SquareControlHandler<'a> {
    fn push(
        &mut self,
        moveable: bool,
        can_capture: bool,
        params: &MoveTestParams,
        dest_x: u8,
        dest_y: u8,
        _: &Square,
        _: Option<Piece>
    ) -> bool {
        if !can_capture { return false; }

        let index = dest_y as usize * 8 + dest_x as usize;

        let mut lowest_controller_value_negpos = self.temp_arr[index];
        {
            let r = lowest_controller_value_negpos.round();
            if r != lowest_controller_value_negpos {
                lowest_controller_value_negpos = r;
            }
        }
        let lowest_controller_value = lowest_controller_value_negpos.abs();

        let candidate_value = evaluate_piece(params.src_piece);
        let candidate_value_negpos = candidate_value * params.src_player.get_multiplier();

        if candidate_value < lowest_controller_value {
            self.temp_arr[index] = candidate_value_negpos;
        } else if candidate_value == lowest_controller_value && candidate_value_negpos != lowest_controller_value_negpos {
            self.temp_arr[index] = candidate_value_negpos + 0.3333;
        }

        return false;
    }
}

#[inline]
pub fn evaluate_piece(piece: Piece) -> f32 {
    PIECE_VALUES[piece as usize] as f32
}

fn evaluate_player(board: &Board, handler: &mut SquareControlHandler, player: Player) -> f32 {

    let ps = board.get_player_state(player);
    let some_values_1 = SOME_VALUES_1[player as usize];

    let mut value: f32 = 0.;

    for Coord(x, y) in ps.piece_locs.iter() {
        let fy = *y as f32;

        if let Square::Occupied(piece, _) = board.get_by_xy(*x, *y) {
            value += evaluate_piece(*piece);
            if *piece == Piece::Pawn {
                value += 0.3 * (some_values_1.0 + some_values_1.1 * fy);
            } else {
                if fy > 0. && fy < 7. {
                    value += 0.75;
                }
            }

            fill_src(&MoveTestParams {
                src_x: *x as i8,
                src_y: *y as i8,
                src_piece: *piece,
                src_player: player,
                can_capture_king: true,
                board: &board
            }, handler);
        }
    }
    value * player.get_multiplier()
}

fn round_eval(v: f32) -> f32 {
    (v * 100.).round() / 100.
}

pub fn evaluate(board: &Board, temp_arr: &mut [f32; 64]) -> f32 {
    clear_square_control(temp_arr);

    let mut handler = SquareControlHandler { temp_arr };
    let white_eval = evaluate_player(board, &mut handler, Player::White);
    let black_eval = evaluate_player(board, &mut handler, Player::Black);
    
    round_eval(0.2 * get_white_square_control(handler.temp_arr) + white_eval + black_eval)
}

pub fn add_captures_to_evals(
    m: &mut MoveList,
    start: usize,
    end_exclusive: usize,
) {
    m.write_evals(start, end_exclusive, |m| {
        let mut score = m.get_eval();
        if let MoveDescription::Capture(_, _, dest_sq_index) = m.get_description() {
            if let Some((_, BeforeAfterSquares(Square::Occupied(before_piece, _), Square::Occupied(after_piece, _)))) = m.0[*dest_sq_index as usize] {
                score += evaluate_piece(before_piece) - evaluate_piece(after_piece);
            }
        }
        score
    });
}

pub fn add_aggression_to_evals(
    board: &Board,
    m: &mut MoveList,
    start: usize,
    end_exclusive: usize,
    temp_arr: &mut [f32; 64],
    temp_ml: &mut MoveList
) {
    clear_square_control(temp_arr);
    get_white_square_control(temp_arr);

    let mut handler = PushToMoveListHandler { move_list: temp_ml };
    m.write_evals(start, end_exclusive, |m| {
        let mut score = m.get_eval();
        for sq_holder in m.get_squares().iter() {
            if let Some((Coord(x, y), BeforeAfterSquares(_, Square::Occupied(after_piece, after_player)))) = sq_holder {
                let min_controlling_value_negpos = temp_arr[*y as usize * 8 + *x as usize];
                if min_controlling_value_negpos != NO_CONTROL_VAL && min_controlling_value_negpos.signum() != after_player.get_multiplier() { continue; } 

                let params = MoveTestParams {
                    src_x: *x as i8,
                    src_y: *y as i8,
                    src_piece: *after_piece,
                    src_player: *after_player,
                    can_capture_king: true,
                    board
                };

                handler.move_list.write_index = 0;
                fill_src(&params, &mut handler);

                for i in 0..handler.move_list.write_index {
                    if let MoveSnapshot(sqs, _, MoveDescription::Capture(_, _, dest_sq_index)) = handler.move_list.get_v()[i] {
                        if let Some((_, BeforeAfterSquares(Square::Occupied(attacked_piece, attacked_player), _))) = sqs[dest_sq_index as usize] {
                            if attacked_player != *after_player {
                                score += evaluate_piece(attacked_piece) * 0.33;
                            }
                        }
                    }
                }
            }
        }
        score
    });
}
