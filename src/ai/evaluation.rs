use super::super::game::entities::*;
use super::super::game::coords::*;
use super::super::game::board::*;
use super::super::game::move_list::*;

/// Precondition: Pawn = 0, Rook, Knight, Bishop, Queen, King
static PIECE_VALUES: [f32; 6] = [
    1., 5., 3., 3., 9., 0.
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

pub fn sort_subset_captures(m: &mut MoveList, start: usize, end_exclusive: usize) {
    m.write_evals(start, end_exclusive, |m| {
        if let MoveDescription::Capture(_, _, dest_sq_index) = m.2 {
            if let Some((_, BeforeAfterSquares(Square::Occupied(before_piece, _), Square::Occupied(after_piece, _)))) = m.0[dest_sq_index as usize] {
                evaluate_piece(before_piece)
            } else {
                // FIXME Panic instead
                crate::console_error!("Unexpected bad move description");
                0.
            }
        } else {
            0.
        }
    });
    m.sort_subset_by_eval(start, end_exclusive);
}

