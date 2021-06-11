use super::super::game::entities::*;
use super::super::game::coords::*;
use super::super::game::board::*;
use super::super::game::move_list::*;
use super::super::game::basic_move_test::*;

/// Precondition: Pawn = 0, Rook, Knight, Bishop, Queen, King
static PIECE_VALUES: [f32; 6] = [
    1., 5., 3., 3., 9., 18.
];

fn evaluate_piece(piece: Piece) -> f32 {
    PIECE_VALUES[piece as usize]
}

fn evaluate_player(board: &Board, neg_one_if_white_else_one: f32, seven_if_white_else_zero: f32, ps: &PlayerState) -> f32 {
    let mut value: f32 = 0.;
    for Coord(x, y) in ps.piece_locs.iter() {
        let fy = *y as f32;

        if let Square::Occupied(piece, _) = board.get_by_xy(*x, *y) {
            let unadvanced = (3.5 - fy).abs();
            value += evaluate_piece(piece);
            value += 0.3 * match piece {
                Piece::Pawn => {
                    seven_if_white_else_zero + neg_one_if_white_else_one * fy
                },
                _ => {
                    3.5 - unadvanced
                }
            };
        }
    }
    if ps.castled_somewhere { value += 1.5; }
    value * neg_one_if_white_else_one as f32 * -1.
}

pub fn evaluate(board: &Board) -> f32 {
    let white_s = &board.get_player_state(Player::White);
    let black_s = &board.get_player_state(Player::Black);
    evaluate_player(board, 1., 0., black_s) + evaluate_player(board, -1., 7., white_s)
}

pub fn sort_moves_by_aggression(board: &Board, m: &mut MoveList, start: usize, end_exclusive: usize, temp_ml: &mut MoveList) {
    m.write_evals(start, end_exclusive, |m| {
        let mut score = 0.0f32;

        if let MoveDescription::Capture(_, _, dest_sq_index) = m.2 {
            if let Some((_, BeforeAfterSquares(Square::Occupied(before_piece, _), Square::Occupied(_, _)))) = m.0[dest_sq_index as usize] {
                score += evaluate_piece(before_piece)
            }
        };

        for sq_holder in m.0.iter() {
            if let Some((Coord(x, y), BeforeAfterSquares(_, Square::Occupied(after_piece, after_player)))) = sq_holder {
                temp_ml.write_index = 0;
                BasicMoveTest::fill_src(*x, *y, *after_piece, *after_player, board, true, temp_ml);
                for i in 0..temp_ml.write_index {
                    if let MoveSnapshot(sqs, _, MoveDescription::Capture(_, _, dest_sq_index)) = temp_ml.get_v()[i] {
                        if let Some((_, BeforeAfterSquares(Square::Occupied(attacked_piece, attacked_player), _))) = sqs[dest_sq_index as usize] {
                            if attacked_player != *after_player {
                                score += evaluate_piece(attacked_piece) * 0.1;
                            }
                        }
                    }
                }
            }
        }

        score
    });
    m.sort_subset_by_eval(start, end_exclusive);
}

