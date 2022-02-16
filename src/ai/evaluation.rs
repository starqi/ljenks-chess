use std::cmp::{max, min};
use super::super::*;
use super::super::game::entities::*;
use super::super::game::bitboard::*;
use super::super::game::board::*;
use super::super::game::move_test::*;
use super::super::game::move_list::*;

/// Matches `Piece` enum number
static PIECE_VALUES: [i32; 6] = [
    100, 500, 300, 300, 900, 1000
];

static PAWN_PUSH_BONUS: i32 = 10;
static MIN_MATERIAL_FOR_PAWN_EVAL: i32 = 2500;
static CASTLE_BONUS: i32 = 50;
static MOVE_ORDER_ATTACK_BONUS: i32 = 30;
static MOVE_ORDER_CASTLE_VAL: i32 = 50;
static MOVE_ORDER_CAPTURE_MIN_VAL: i32 = 100;
static MOVE_ORDER_MOB_SQ_VAL: i32 = 3;
static MOVE_ORDER_MOB_CENTER_SQ_BONUS: i32 = 3;
static PIECE_VALUE_BOUND_FOR_CONTROL: i32 = 10;
static CONTROL_SURPLUS_TO_EVAL_LSHIFT: i32 = 8;

/// Matches `Piece` enum number
static PIECE_VALUES_FOR_CONTROL: [i32; 6] = [
    1, 5, 3, 3, 9, 10
];

static PAWN_Y_CONSTANTS: [(i32, i32); 2] = [(6, -1), (-1, 1)];

// TODO Array
#[inline]
pub fn get_base_sq_worth_white(x: i32, y: i32) -> i32 {
    if y <= 1 { 100 }
    else if y <= 2 { 75 }
    else if y >= 6 { 10 }
    else { 10 + ((35 - (35 - x * 10).abs()) << 2) }
}

static PIECE_VALUE_TO_CONTROL_MULTIPLIER: [i32; 11] = [
    0, 9, 0, 3, 0, 2, 0, 0, 0, 0, 1
];

#[inline]
pub fn evaluate_piece(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize] as i32
}

#[inline]
pub fn evaluate_piece_for_control(piece: Piece) -> i32 {
    PIECE_VALUES_FOR_CONTROL[piece as usize] as i32
}

/// Precondition: `prepared_af_boards` is filled in with attacked from map
fn evaluate_player(board: &Board, player: Player) -> i32 {

    let ps = board.get_player_state(player);
    let mut value: i32 = 0;

    // Count material
    let mut piece_locs_copy = ps.piece_locs;
    piece_locs_copy.consume_loop_indices(|index| {
        if let Square::Occupied(piece, _) = board.get_by_index(index) {
            value += evaluate_piece(*piece);
        }
    });

    // Reward pawn push in later stages of game
    if value <= MIN_MATERIAL_FOR_PAWN_EVAL {
        let pawn_y_consts = PAWN_Y_CONSTANTS[player as usize];
        let mut piece_locs_copy = ps.piece_locs;
        piece_locs_copy.consume_loop_indices(|index| {
            let coord = FastCoord(index).to_coord();
            if let Square::Occupied(piece, _) = board.get_by_index(index) {
                let is_pawn_mask = -((*piece == Piece::Pawn) as i32);
                value += is_pawn_mask & ((pawn_y_consts.0 + pawn_y_consts.1 * (coord.1 as i32)) * PAWN_PUSH_BONUS);
            }
        });
    }

    value += -(ps.is_castled as i32) & CASTLE_BONUS;
    value * player.multiplier()
}

fn calculate_control(board: &Board, prepared_af_boards: &mut AttackFromBoards) -> i32 {

    board.rewrite_af_boards(prepared_af_boards);

    let mut white_square_surplus: i32 = 0;
    for y in 0..8 {
        for x in 0..8 {
            let b = prepared_af_boards.data[y * 8 + x];
            let mut lowest_attacker_worth: [i32; 2] = [PIECE_VALUE_BOUND_FOR_CONTROL, PIECE_VALUE_BOUND_FOR_CONTROL];

            let mut b2 = b;
            b2.consume_loop_indices(|index| {
                match board.get_by_index(index) {
                    Square::Occupied(attacking_piece, attacking_player) => {
                        let value = evaluate_piece_for_control(*attacking_piece);
                        let ref mut lowest_ref = lowest_attacker_worth[*attacking_player as usize];
                        *lowest_ref = min(*lowest_ref, value);
                    },
                    Square::Blank => panic!("Unexpected empty square when attacker is expected")
                };
            });

            let one_or_neg_one_or_zero = (lowest_attacker_worth[1] - lowest_attacker_worth[0]).signum();
            let zero_if_white = (one_or_neg_one_or_zero != 1) as i32;
            let square_worth = get_base_sq_worth_white(x as i32, zero_if_white * 7 + one_or_neg_one_or_zero * (y as i32)) *
                PIECE_VALUE_TO_CONTROL_MULTIPLIER[lowest_attacker_worth[zero_if_white as usize] as usize];
            // TODO Two arrays for black and white
            white_square_surplus += one_or_neg_one_or_zero * square_worth;
        }
    }

    white_square_surplus >> CONTROL_SURPLUS_TO_EVAL_LSHIFT
}

pub fn evaluate(board: &Board, prepared_af_boards: &mut AttackFromBoards) -> i32 {
    let white_eval = evaluate_player(board, Player::White);
    let black_eval = evaluate_player(board, Player::Black);
    
    white_eval + black_eval + calculate_control(board, prepared_af_boards)
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
                    score += max(evaluate_piece(*curr_dest_piece) - evaluate_piece(*dragged_piece), MOVE_ORDER_CAPTURE_MIN_VAL);
                }
            }
        }
        score
    });
}

/// Precondition: Move list is the current player's moves
pub fn add_mobility_to_evals(
    board: &Board,
    m: &mut MoveList,
    start: usize,
    end_exclusive: usize,
) {
    let opp_state = board.get_player_state(board.get_player_with_turn().other_player());

    m.write_evals(start, end_exclusive, |m| {
        let mut score = m.eval();

        if let MoveDescription::NormalMove(_from_coord, _to_coord) = m.description() {
            if let Square::Occupied(src_piece, src_player) = board.get_by_index(_from_coord.value()) {
                let atks = board.get_imaginary_pseudo_move_at(*_to_coord, *src_piece, *src_player);
                score += atks.pop_count() as i32 * MOVE_ORDER_MOB_SQ_VAL;

                let piece_atks = Bitboard(atks.0 & opp_state.piece_locs.0);
                score += -((piece_atks.0 != 0) as i32) & MOVE_ORDER_ATTACK_BONUS;

                let mut important_sq_atks = Bitboard(atks.0 & (BITBOARD_PRESETS.central_squares.0 | BITBOARD_PRESETS.opponent_squares[*src_player as usize].0));
                score += important_sq_atks.consume_pop_count() as i32 * MOVE_ORDER_MOB_CENTER_SQ_BONUS;
            }
        } else if let MoveDescription::Castle(_) = m.description() {
            score += MOVE_ORDER_CASTLE_VAL;
        }

        score
    });
}

#[cfg(test)]
mod test {

    use super::*;

    #[ignore]
    #[test]
    fn move_mob_eyeball_test() {
        let mut board = Board::new();
        board.set_uniform_row_test(2, Square::Blank);
        board.set_uniform_row_test(6, Square::Blank);

        let mut ml = MoveList::new(50);
        board.get_pseudo_moves_at(FastCoord::from_xy(3, 7), &mut ml);
        let write_index = ml.write_index;
        add_mobility_to_evals(&board, &mut ml, 0, write_index);
        board.print_move_list(&ml, 0, ml.write_index);
    }

    #[ignore]
    #[test]
    fn control_eyeball_test() {
        let mut board = Board::empty();
        let mut af = AttackFromBoards::new();
        board.set_by_file_rank_test('d', 4, Square::Occupied(Piece::Queen, Player::White));
        println!("{}", calculate_control(&board, &mut af));
    }

    #[test]
    fn basic_square_control() {
        let mut board = Board::new();
        board.set_uniform_row_test(2, Square::Blank);
        board.set_uniform_row_test(7, Square::Blank);
        let mut af = AttackFromBoards::new();

        let mut white_control_surplus = calculate_control(&board, &mut af);
        assert_eq!(white_control_surplus, 0);

        board.set_by_file_rank_test('d', 1, Square::Blank);
        board.set_by_file_rank_test('a', 1, Square::Blank);
        white_control_surplus = calculate_control(&board, &mut af);
        println!("a {}", white_control_surplus);
        assert!(white_control_surplus < 0);

        board.set_by_file_rank_test('d', 8, Square::Blank);
        board.set_by_file_rank_test('a', 8, Square::Blank);
        board.set_by_file_rank_test('g', 8, Square::Blank);
        board.set_by_file_rank_test('b', 8, Square::Blank);
        white_control_surplus = calculate_control(&board, &mut af);
        println!("b {}", white_control_surplus);
        assert!(white_control_surplus > 0);
    }
}
