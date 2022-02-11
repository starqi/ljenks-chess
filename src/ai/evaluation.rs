use std::cmp::min;
use super::super::game::entities::*;
use super::super::game::coords::*;
use super::super::game::board::*;
use super::super::game::move_test::*;
use super::super::game::move_list::*;

static PIECE_VALUE_BOUND: i32 = 1000;

static PIECE_VALUES: [i32; 6] = [
    100, 500, 300, 300, 900, 100
];

#[inline]
pub fn get_square_worth_white(x: i32, y: i32) -> i32 {
    if y <= 1 { 100 }
    else if y >= 6 { 10 }
    else { 150 * (350 - (350 - x * 100).abs() + 40) / 400 }
}

#[inline]
pub fn evaluate_piece(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize] as i32
}

static PAWN_Y_CONSTANTS: [(i32, i32); 2] = [(6, -1), (-1, 1)];

/// Precondition: `prepared_af_boards` is filled in with attacked from map
fn evaluate_player(board: &Board, player: Player) -> i32 {

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

    value * player.multiplier()
}

fn calculate_white_control(board: &Board, prepared_af_boards: &mut AttackFromBoards) -> i32 {

    board.rewrite_af_boards(prepared_af_boards);

    let mut white_square_surplus: i32 = 0;
    for y in 0..8 {
        for x in 0..8 {
            let b = prepared_af_boards.data[y * 8 + x];
            let mut lowest_attacker_worth: [i32; 2] = [PIECE_VALUE_BOUND, PIECE_VALUE_BOUND];

            let mut b2 = b;
            b2.consume_loop_indices(|index| {
                match board.get_by_index(index) {
                    Square::Occupied(attacking_piece, attacking_player) => {
                        let value = evaluate_piece(*attacking_piece);
                        let ref mut lowest_ref = lowest_attacker_worth[*attacking_player as usize];
                        *lowest_ref = min(*lowest_ref, value);
                    },
                    Square::Blank => panic!("Unexpected empty square when attacker is expected")
                };
            });

            let one_or_neg_one_or_zero = (lowest_attacker_worth[1] - lowest_attacker_worth[0]).signum();
            let zero_if_white = (one_or_neg_one_or_zero != 1) as i32;
            white_square_surplus += one_or_neg_one_or_zero * get_square_worth_white(x as i32, zero_if_white * 7 + one_or_neg_one_or_zero * (y as i32));
        }
    }

    white_square_surplus >> 3
}

pub fn evaluate(board: &Board, prepared_af_boards: &mut AttackFromBoards) -> i32 {
    let white_eval = evaluate_player(board, Player::White);
    let black_eval = evaluate_player(board, Player::Black);
    
    white_eval + black_eval + calculate_white_control(board, prepared_af_boards)
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn basic_square_control() {
        let mut board = Board::new();
        board.set_uniform_row_test(2, Square::Blank);
        board.set_uniform_row_test(7, Square::Blank);
        let mut af = AttackFromBoards::new();

        let mut white_control_surplus = calculate_white_control(&board, &mut af);
        assert_eq!(white_control_surplus, 0);

        board.set_test('d', 1, Square::Blank);
        board.set_test('a', 1, Square::Blank);
        white_control_surplus = calculate_white_control(&board, &mut af);
        println!("a {}", white_control_surplus);
        assert!(white_control_surplus < 0);

        board.set_test('d', 8, Square::Blank);
        board.set_test('a', 8, Square::Blank);
        board.set_test('g', 8, Square::Blank);
        board.set_test('b', 8, Square::Blank);
        white_control_surplus = calculate_white_control(&board, &mut af);
        println!("b {}", white_control_surplus);
        assert!(white_control_surplus > 0);
    }
}
