use super::super::game::entities::*;
use super::super::game::coords::*;
use super::super::game::board::*;
use super::super::game::move_list::*;

static PIECE_VALUES: [i32; 6] = [
    100, 500, 300, 300, 900, 100
];

#[derive(Copy, Clone, PartialEq, Eq)]
enum Value { NoValue, Value(i32) }

// TODO Extract
pub struct ValueBoard {
    data: [Value; 64]
}

impl ValueBoard {

    pub fn new() -> ValueBoard {
        ValueBoard { data: [Value::NoValue; 64] }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.data.fill(Value::NoValue);
    }

    #[inline]
    pub fn set_by_index(&mut self, index: usize, value: Value) {
        self.data[index] = value;
    }

    #[inline]
    pub fn get_by_index(&self, index: usize) -> &Value {
        &self.data[index]
    }
}

#[inline]
pub fn get_square_worth_white(x: i32, y: i32) -> i32 {
    if y <= 1 { 100 }
    else if y >= 6 { 10 }
    else { 150 * (350 - (350 - x * 100).abs() + 40) / 400 }
}

#[inline]
pub fn get_square_worth_black(x: i32, y: i32) -> i32 {
    if y >= 6 { 100 }
    else if y <= 1 { 10 }
    else { 150 * (350 - (350 - x * 100).abs() + 40) / 400 }
}

#[inline]
pub fn evaluate_piece(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize] as i32
}

static PAWN_Y_CONSTANTS: [(i32, i32); 2] = [(6, -1), (-1, 1)];

fn evaluate_player(
    board: &Board,
    player: Player,
    temp_ml: &mut MoveList,
    result: &mut ValueBoard
) -> i32 {

    let ps = board.get_player_state(player);
    let pawn_y_consts = PAWN_Y_CONSTANTS[player as usize];
    let mut value: i32 = 0;

    let mut piece_locs_copy = ps.piece_locs;
    piece_locs_copy.consume_loop_indices(|index| {
        let coord = FastCoord(index).to_coord();

        if let Square::Occupied(piece, _) = board.get_by_index(index) {
            value += evaluate_piece(*piece);

            // Reward pawn push
            let is_pawn_mask = -((*piece == Piece::Pawn) as i32);
            value += is_pawn_mask & ((pawn_y_consts.0 + pawn_y_consts.1 * (coord.1 as i32)) * 30);
        }
    });

    /*
    temp_ml.write_index = 0;
    // FIXME Need to be able to choose the player
    board.get_pseudo_moves(temp_ml);

    for m in temp_ml.v() {
        if let MoveWithEval(MoveDescription::NormalMove(_from_coord, _to_coord), _) = m {
            FIXME
        }
    }
    */

    value * player.multiplier()
}

pub fn evaluate(board: &Board, temp_ml: &mut MoveList, result: &mut ValueBoard) -> i32 {
    result.reset();

    let white_eval = evaluate_player(board, Player::White, temp_ml, result);
    let black_eval = evaluate_player(board, Player::Black, temp_ml, result);
    
    white_eval + black_eval
    //round_eval(0.2 * get_white_square_control(handler.temp_arr) + white_eval + black_eval)
}

pub fn add_captures_to_evals(
    board: &Board,
    m: &mut MoveList,
    start: usize,
    end_exclusive: usize,
) {
    m.write_evals(start, end_exclusive, |m| {
        let mut score = m.eval();
        if let MoveDescription::NormalMove(_from_coord, _to_coord) = m.description() {
            if let Square::Occupied(curr_dest_piece, _) = board.get_by_index(_to_coord.value()) {
                if let Square::Occupied(dragged_piece, _) = board.get_by_index(_from_coord.value()) {
                    score += evaluate_piece(*curr_dest_piece) - evaluate_piece(*dragged_piece);
                }
            }
        }
        score
    });
}

/*
pub fn add_aggression_to_evals(
    board: &Board,
    m: &mut MoveList,
    start: usize,
    end_exclusive: usize,
    temp_ml: &mut MoveList
) {
    let curr_player = board.get_player_with_turn();
    let mut handler = PushToMoveListHandler { move_list: temp_ml };
    m.write_evals(start, end_exclusive, |m| {
        let mut score = m.eval();
        if let MoveDescription::NormalMove(_from_coord, _to_coord) = m.description() {
            if let Square::Occupied(dragged_piece, _) = board.get_by_index(_from_coord.value()) {

                let to_coord = _to_coord.to_coord();
                let params = MoveTestParams {
                    src_x: to_coord.0 as i8,
                    src_y: to_coord.1 as i8,
                    src_piece: *dragged_piece,
                    src_player: curr_player,
                    can_capture_king: true,
                    board
                };

                handler.move_list.write_index = 0;
                fill_src(&params, &mut handler);

                for i in 0..handler.move_list.write_index {
                    if let MoveWithEval(MoveDescription::NormalMove(_from_coord2, _to_coord2), _) = handler.move_list.v()[i] {
                        if let Square::Occupied(target_piece, target_player) = board.get_by_index(_to_coord2.value()) {
                            if *target_player != curr_player {
                                score += evaluate_piece(*target_piece) * 0.33;
                            }
                        }
                    }
                }
            }
        }
        score
    });
}
*/
