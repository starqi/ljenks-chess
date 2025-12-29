use std::cmp::{max, min};
use super::super::*;
use super::super::game::entities::*;
use super::super::game::bitboard::*;
use super::super::game::board::*;
use super::super::game::move_test::*;
use super::super::game::move_list::*;

/// Matches `Piece` enum number
static PIECE_VALUES: [i32; 6] = [
    100, 500, 300, 300, 900, 0 // Pretty sure king can be zero for this engine
];

const PAWN_PUSH_BONUS: i32 = 10;
pub const MIN_MATERIAL_FOR_PAWN_EVAL: i32 = 2500;
const CASTLE_BONUS: i32 = 50;
const MOVE_ORDER_ATTACK_BONUS: i32 = 100;
const MOVE_ORDER_CASTLE_VAL: i32 = 80;
const MOVE_ORDER_CAPTURE_BASE_VAL: i32 = 100;
const MOVE_ORDER_MOB_SQ_VAL: i32 = 10;
const MOVE_ORDER_MOB_CENTER_SQ_BONUS: i32 = 10;
const PIECE_VALUE_BOUND_FOR_CONTROL: i32 = 10;
const CONTROL_SURPLUS_TO_EVAL_DOWNSCALE_SHIFT: i32 = 8;
const DEFENDED_PAWN_BONUS: i32 = 4;

// [Balance between non-material evals]
// 1 key square (100) * PIECE_VALUE_TO_CONTROL_MULTIPLIER (30) / 256 -> ~ 11 cp.
// 5 defended pawns (most of the board) * DEFENDED_PAWN_BONUS (4) -> 20 cp.
// Positional play is extremely sensitive to these values in practice.

/// Index is `Piece` enum number.
/// The higher the output, the worse the defender, e.g. 10 = king, 9 = queen.
static PIECE_TO_CONTROL_BADNESS: [i32; 6] = [
    1, 5, 3, 3, 9, 10
];

static PAWN_Y_CONSTANTS: [(i32, i32); 2] = [(6, -1), (-1, 1)];

/// Controlling enemy territory = good, controlling own territory = useless, control center = good.
static POSITIONAL_SQUARE_WORTH_WHITE: [i32; 64] = [
    90,    90,  100,  100,  100,  100,   90,   90,
    90,    90,  100,  100,  100,  100,   90,   90,
    70,    80,   90,   90,   90,   90,   80,   70,
    70,    80,   90,  100,  100,   90,   80,   70,
    50,    50,   60,   70,   70,   60,   50,   50,
    30,    30,   40,   40,   40,   40,   30,   30,
    10,    10,   10,   10,   10,   10,   10,   10,
    10,    10,   10,   10,   10,   10,   10,   10,
];

fn get_positional_sq_worth_white(x: i32, y: i32) -> i32 {
    POSITIONAL_SQUARE_WORTH_WHITE[(y * 8 + x) as usize]
}

/// Maps `PIECE_TO_CONTROL_BADNESS` number as index to higher-the-better control score. 
static PIECE_VALUE_TO_CONTROL_MULTIPLIER: [i32; 11] = [
    // I think I was setting queen control mult to 0 to stop sending the queen out?
    0, 30, 0, 30, 0, 30, 0, 0, 0, 0, 0
];

#[inline]
pub fn evaluate_piece(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize] as i32
}

#[inline]
pub fn piece_to_control_badness(piece: Piece) -> i32 {
    PIECE_TO_CONTROL_BADNESS[piece as usize] as i32
}

pub fn count_material(board: &Board, player: Player) -> i32 {
    let mut value: i32 = 0;

    let ps = board.get_player_state(player);
    let mut piece_locs_copy = ps.piece_locs;
    piece_locs_copy.consume_loop_indices(|index| {
        if let Square::Occupied(piece, _) = board.get_by_index(index) {
            value += evaluate_piece(*piece);
        }
    });
    value
}

/// Precondition: `prepared_af_boards` is filled in with attacked from map
fn evaluate_player(board: &Board, player: Player) -> i32 {

    let mut value = count_material(board, player);

    let ps = board.get_player_state(player);

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

    let defended_pawn_count = get_pawndefended_pawn_count(board, player);
    value += defended_pawn_count as i32 * DEFENDED_PAWN_BONUS;

    value += -(ps.is_castled as i32) & CASTLE_BONUS;
    value * player.multiplier()
}

// TODO Review general eval performance reqs... Turn it off and check NPS

/// Returns how much more white controls all squares than black, where control belongs to the side controlling with a lower valued piece.
/// A square is scaled by position (favouring center, enemy side) and piece value (lower better).
fn calculate_control(board: &Board, prepared_af_boards: &mut AttackFromBoards) -> i32 {

    board.rewrite_af_boards_both_players(prepared_af_boards);

    let mut white_square_surplus: i32 = 0;
    for y in 0..8 {
        for x in 0..8 {
            let b = prepared_af_boards.data[y * 8 + x];
            let mut lowest_attacker_worth: [i32; 2] = [PIECE_VALUE_BOUND_FOR_CONTROL, PIECE_VALUE_BOUND_FOR_CONTROL];

            let mut b2 = b;
            b2.consume_loop_indices(|index| {
                match board.get_by_index(index) {
                    Square::Occupied(attacking_piece, attacking_player) => {
                        let badness = piece_to_control_badness(*attacking_piece);
                        let ref mut lowest_ref = lowest_attacker_worth[*attacking_player as usize];
                        *lowest_ref = min(*lowest_ref, badness);
                    },
                    Square::Blank => panic!("Unexpected empty square when attacker is expected")
                };
            });

            let one_or_neg_one_or_zero = (lowest_attacker_worth[1] - lowest_attacker_worth[0]).signum();
            if one_or_neg_one_or_zero != 0 { // If one side controls more than another
                let zero_if_white_controlled = (one_or_neg_one_or_zero != 1) as i32;
                let square_worth = get_positional_sq_worth_white(
                    x as i32,
                    // Branchless way: If white controlled, normal coordinates. If black controlled, 7 - y
                    zero_if_white_controlled * 7 + one_or_neg_one_or_zero * (y as i32)
                ) * PIECE_VALUE_TO_CONTROL_MULTIPLIER[lowest_attacker_worth[zero_if_white_controlled as usize] as usize];
                // TODO (???) Two arrays for black and white
                white_square_surplus += one_or_neg_one_or_zero * square_worth;
            }
        }
    }

    white_square_surplus >> CONTROL_SURPLUS_TO_EVAL_DOWNSCALE_SHIFT // Chess way of multiplying by (1/256)
}

/// For a player, gets number of pawns defended by another pawn, pawns counted once.
/// For eval purposes, does not count attackers on top or bottom rank, pointless edge case.
pub fn get_pawndefended_pawn_count(board: &Board, player: Player) -> u8 {
    let ps = board.get_player_state(player);
    let mut piece_locs_clone = ps.piece_locs.clone();
    let mut player_pawn_bb = Bitboard(0);
    piece_locs_clone.consume_loop_indices(|index| {
        if let Square::Occupied(Piece::Pawn, _) = board.get_by_index(index) {
            player_pawn_bb.set_index(index);
        }
    });

    const NOT_A_FILE: u64 = 0b00000000_01111111_01111111_01111111_01111111_01111111_01111111_00000000;
    const NOT_H_FILE: u64 = 0b00000000_11111110_11111110_11111110_11111110_11111110_11111110_00000000;

    // Union right and left attacks
    let attacks = if player == Player::White {
        ((player_pawn_bb.0 & NOT_H_FILE) << 7) | ((player_pawn_bb.0 & NOT_A_FILE) << 9)
    } else {
        ((player_pawn_bb.0 & NOT_H_FILE) >> 9) | ((player_pawn_bb.0 & NOT_A_FILE) >> 7)
    };

    let defended_bb = Bitboard(player_pawn_bb.0 & attacks);
    defended_bb.pop_count() as u8
}

pub fn evaluate(board: &Board, prepared_af_boards: &mut AttackFromBoards) -> i32 {
    let white_eval = evaluate_player(board, Player::White);
    let black_eval = evaluate_player(board, Player::Black);
    
    white_eval + black_eval + calculate_control(board, prepared_af_boards)
}

// [Balance between move ordering evals]
// Always add base 100 for attacks and captures, more for capturing an extra piece.
// Attacking ~4 important squares -> 80, almost as important as a basic attack or capture.
// In end game, when shuffling pawns and pieces, none of the current criteria matter, then off LMR...
// Remember this matters very much, bad evals here will allow LMR to prune. 

pub fn add_captures_to_evals(
    board: &Board,
    m: &mut MoveList,
    start: usize,
    end_exclusive: usize,
) {
    m.write_evals(start, end_exclusive, |m| {
        let mut score = m.ordering_score();
        if let MoveDescription::NormalMove(_from_coord, _to_coord, _) = m.description() {
            if let Square::Occupied(curr_dest_piece, _) = board.get_by_index(_to_coord.value()) {
                if let Square::Occupied(dragged_piece, _) = board.get_by_index(_from_coord.value()) {
                    score += max(evaluate_piece(*curr_dest_piece) - evaluate_piece(*dragged_piece), 0);
                    score += MOVE_ORDER_CAPTURE_BASE_VAL;
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
        let mut score = m.ordering_score();

        if let MoveDescription::NormalMove(_from_coord, _to_coord, _) = m.description() {
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
        let mut board = Board::with_kings_only();
        let mut af = AttackFromBoards::new();
        board.set_by_file_rank_test('d', 4, Square::Occupied(Piece::Queen, Player::White));
        println!("{}", calculate_control(&board, &mut af));
    }

    #[test]
    fn basic_square_control() {
        // TODO IMMEDIATE Generate routine to visualize control of each square and pass this test

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

    #[test]
    fn standard_pawndefended_some_chain_some_not() {
        let mut board = Board::new();
        board.set_by_file_rank_test('e', 3, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('d', 4, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('c', 5, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('c', 4, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('c', 6, Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(3, get_pawndefended_pawn_count(&board, Player::White));
    }

    #[test]
    fn long_pawn_chain_pawndefended_pawn_count() {
        let mut board = Board::with_kings_only();
        board.set_by_file_rank_test('h', 4, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('g', 3, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('f', 2, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('e', 3, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('d', 4, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('c', 5, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('b', 6, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('a', 7, Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(7, get_pawndefended_pawn_count(&board, Player::White));
    }

    #[test]
    fn pawndefended_no_pawns_on_board() {
        let mut board = Board::new();
        assert_eq!(0, get_pawndefended_pawn_count(&board, Player::White));
        assert_eq!(0, get_pawndefended_pawn_count(&board, Player::Black));
    }

    #[test]
    fn pawns_on_board_edges() {
        let mut board = Board::new();
        board.set_by_file_rank_test('a', 2, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('b', 3, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('h', 7, Square::Occupied(Piece::Pawn, Player::Black));
        board.set_by_file_rank_test('g', 6, Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(1, get_pawndefended_pawn_count(&board, Player::White));
        assert_eq!(1, get_pawndefended_pawn_count(&board, Player::Black));
    }

    #[test]
    fn mixed_players_pawns() {
        let mut board = Board::new();
        board.set_by_file_rank_test('d', 4, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('c', 5, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('e', 3, Square::Occupied(Piece::Pawn, Player::Black));
        board.set_by_file_rank_test('f', 2, Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(1, get_pawndefended_pawn_count(&board, Player::White));
        assert_eq!(1, get_pawndefended_pawn_count(&board, Player::Black));
    }

    #[test]
    fn isolated_pawns() {
        let mut board = Board::new();
        board.set_by_file_rank_test('c', 4, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('f', 5, Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(0, get_pawndefended_pawn_count(&board, Player::White));
        assert_eq!(0, get_pawndefended_pawn_count(&board, Player::Black));
    }

    #[test]
    fn overlapping_pawn_defenses() {
        let mut board = Board::new();
        board.set_by_file_rank_test('d', 4, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('c', 5, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('e', 5, Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(2, get_pawndefended_pawn_count(&board, Player::White));
    }
}

