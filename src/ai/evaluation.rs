use std::cmp::min;
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
const MOVE_ORDER_ATTACK_BONUS: i32 = 50;
const MOVE_ORDER_CASTLE_VAL: i32 = 80;
const MOVE_ORDER_MOB_SQ_VAL: i32 = 1;
const PIECE_VALUE_BOUND_FOR_CONTROL: i32 = 10;
const CONTROL_SURPLUS_TO_EVAL_DOWNSCALE_SHIFT: i32 = 8;
const DEFENDED_PAWN_BONUS: i32 = 4;

// [Non-material board eval]
// 1 key square (100) * CONTROL_BADNESS_TO_CONTROL_MULTIPLIER (30) / 256 -> ~ 11 cp.
// 5 defended pawns (most of the board) * DEFENDED_PAWN_BONUS (4) -> 20 cp.
// Positional play is extremely sensitive to these values in practice.
// For performance, don't compute non-material eval if material is drastically different.

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

// Pawn = 0, Rook, Knight, Bishop, Queen, King
static PIECE_TO_MOB_MULTIPLIER: [i32; 6] = [
    50, 20, 30, 30, 10, 0
];

/// (Deprecated)
/// Index is `Piece` enum number.
/// The higher the output, the worse the defender, e.g. 10 = king, 9 = queen.
static PIECE_TO_CONTROL_BADNESS: [i32; 6] = [
    1, 5, 3, 3, 9, 10
];

/// (Deprecated)
/// Maps `PIECE_TO_CONTROL_BADNESS` number as index to higher-the-better control score. 
static CONTROL_BADNESS_TO_CONTROL_MULTIPLIER: [i32; 11] = [
    0, 50, 0, 30, 0, 30, 0, 0, 0, 10, 0
];

#[inline]
pub fn evaluate_piece(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize] as i32
}

pub fn count_positive_material(board: &Board, player: Player) -> i32 {
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

fn positive_evaluate_player_not_material_mob(board: &Board, player: Player, mut positive_material: i32) -> i32 {
    let ps = board.get_player_state(player);

    // Reward pawn push in later stages of game
    if positive_material <= MIN_MATERIAL_FOR_PAWN_EVAL {
        let pawn_y_consts = PAWN_Y_CONSTANTS[player as usize];
        let mut piece_locs_copy = ps.piece_locs;
        piece_locs_copy.consume_loop_indices(|index| {
            let coord = FastCoord(index).to_coord();
            let is_pawn = matches!(board.get_by_index(index), Square::Occupied(Piece::Pawn, _));
            positive_material += branchless_mask!(is_pawn, (pawn_y_consts.0 + pawn_y_consts.1 * (coord.1 as i32)) * PAWN_PUSH_BONUS);
        });
    }

    let defended_pawn_count = get_pawndefended_pawn_count(board, player);
    positive_material += defended_pawn_count as i32 * DEFENDED_PAWN_BONUS;

    positive_material += branchless_mask!(ps.is_castled, CASTLE_BONUS);
    positive_material
}

/// (Deprecated...)
/// Returns how much more white controls all squares than black, where control belongs to the side controlling with a lower valued piece.
/// A square is scaled by position (favouring center, enemy side) and piece value (lower better).
fn calculate_control(board: &Board, prepared_af_boards: &mut AttackFromBoards) -> i32 {
    // Turning this off -> 28% more NPS

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
                        let badness = PIECE_TO_CONTROL_BADNESS[*attacking_piece as usize];
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
                ) * CONTROL_BADNESS_TO_CONTROL_MULTIPLIER[lowest_attacker_worth[zero_if_white_controlled as usize] as usize];
                // TODO (???) Two arrays for black and white
                white_square_surplus += one_or_neg_one_or_zero * square_worth;
            }
        }
    }

    white_square_surplus >> CONTROL_SURPLUS_TO_EVAL_DOWNSCALE_SHIFT // Chess way of multiplying by (1/256)
}

fn calculate_mobility(board: &Board) -> i32 {
    let mut totals = [0i32; 2];
    for player in [Player::White, Player::Black] {
        let player_idx = player as usize;
        let player_offset = player as i32;

        let ps = board.get_player_state(player);
        let mut piece_locs_copy = ps.piece_locs;
        piece_locs_copy.consume_loop_indices(|index| {
            if let Square::Occupied(piece, _) = board.get_by_index(index) {
                let piece_multiplier = PIECE_TO_MOB_MULTIPLIER[*piece as usize];

                // TOOD Take into account self-captures?
                let attacks = board.get_imaginary_pseudo_move_at(FastCoord(index), *piece, player);
                let mut attacks_copy = attacks;
                attacks_copy.consume_loop_indices(|attack_index| {
                    let attack_coord = FastCoord(attack_index).to_coord();
                    let y = attack_coord.1 as i32;
                    let perspective_y = y + player_offset * (7 - 2 * y); // 7 - y if black
                    // TODO Why is this multiplying? Add? 
                    // TODO Starting to see how piece-square tables provide long term intuition,
                    // and would fit here, since search range can't see the wrongness of bad bishop diagonals, or knights on rim.
                    let pos_worth = get_positional_sq_worth_white(attack_coord.0 as i32, perspective_y);
                    totals[player_idx] += pos_worth * piece_multiplier;
                });
            }
        });
    }
    (totals[0] - totals[1]) >> CONTROL_SURPLUS_TO_EVAL_DOWNSCALE_SHIFT // Chess way of multiplying by (1/256)
}

/// For a player, gets number of pawns defended by another pawn, pawns counted once.
/// For eval purposes, does not count attackers on top or bottom rank, pointless edge case.
fn get_pawndefended_pawn_count(board: &Board, player: Player) -> u8 {
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

/// See [Non-material board eval].
pub fn evaluate(board: &Board) -> i32 {

    let white_pm = count_positive_material(board, Player::White);
    let black_pm = count_positive_material(board, Player::Black);
    let mut e = white_pm - black_pm;
    if e.abs() >= 300 { return e; }

    e += positive_evaluate_player_not_material_mob(board, Player::White, white_pm);
    e -= positive_evaluate_player_not_material_mob(board, Player::Black, black_pm);
    e += calculate_mobility(board);
    e
}

pub fn add_mobility_to_vec(board: &Board, vec: &mut Vec<MoveWithEval>) {
    let player = board.get_player_with_turn();
    let opp_state = board.get_player_state(player.other_player());

    for m in vec.iter_mut() {
        let mut score = m.ordering_score();

        if let MoveDescription::NormalMove(_from_coord, _to_coord, _) = m.description() {
            if let Square::Occupied(src_piece, src_player) = board.get_by_index(_from_coord.value()) {

                // Reduce queen mobility score because it's double rook/bishop mobility.
                let mut mobility_score = branchless_mask!(*src_piece != Piece::Queen, MOVE_ORDER_MOB_SQ_VAL);
                mobility_score += MOVE_ORDER_MOB_SQ_VAL;

                let moves = board.get_imaginary_pseudo_move_at(*_to_coord, *src_piece, *src_player);
                score += moves.pop_count() as i32 * mobility_score;

                let piece_atks = Bitboard(moves.0 & opp_state.piece_locs.0);
                score += branchless_mask!(piece_atks.0 != 0, MOVE_ORDER_ATTACK_BONUS);

                let mut important_sq_moves = Bitboard(moves.0 & (BITBOARD_PRESETS.central_squares.0 | BITBOARD_PRESETS.opponent_squares[*src_player as usize].0));
                score += important_sq_moves.consume_pop_count() as i32 * mobility_score;
            }
        } else if let MoveDescription::Castle(_) = m.description() {
            score += MOVE_ORDER_CASTLE_VAL;
        }

        m.1 = score;
    }
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

        let mut ml = MoveList::new(0);
        board.get_pseudo_moves_at(FastCoord::from_xy(3, 7), &mut ml); // D1, queen

        add_mobility_to_vec(&board, ml.v_unsafe());
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
        let board = Board::new();
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

    #[test]
    fn calculate_mobility_equal_then_blocked() {
        let mut board = Board::with_kings_only();
        board.set_by_file_rank_test('c', 8, Square::Occupied(Piece::Rook, Player::Black));
        board.set_by_file_rank_test('g', 8, Square::Occupied(Piece::Rook, Player::Black));
        board.set_by_file_rank_test('c', 1, Square::Occupied(Piece::Rook, Player::White));
        board.set_by_file_rank_test('g', 1, Square::Occupied(Piece::Rook, Player::White));
        let mut surplus = calculate_mobility(&board);
        assert!(surplus == 0);
        board.set_by_file_rank_test('g', 2, Square::Occupied(Piece::Pawn, Player::White));
        surplus = calculate_mobility(&board);
        assert!(surplus < 0);
        board.set_by_file_rank_test('g', 7, Square::Occupied(Piece::Pawn, Player::Black));
        surplus = calculate_mobility(&board);
        assert!(surplus == 0);
        board.set_by_file_rank_test('g', 2, Square::Occupied(Piece::Pawn, Player::Black));
        board.set_by_file_rank_test('g', 7, Square::Occupied(Piece::Pawn, Player::White));
        surplus = calculate_mobility(&board);
        assert!(surplus == 0);
        board.set_by_file_rank_test('g', 2, Square::Blank);
        surplus = calculate_mobility(&board);
        assert!(surplus > 0);
    }
}

