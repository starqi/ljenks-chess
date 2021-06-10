use std::collections::HashSet;
use super::super::game::entities::*;
use super::super::game::coords::*;
use super::super::game::board::*;

pub fn evaluate_player(board: &Board, neg_one_if_white_else_one: f32, seven_if_white_else_zero: f32, ps: &PlayerState) -> f32 {
    let mut value: f32 = 0.;
    for Coord(x, y) in ps.piece_locs.iter() {
        let fy = *y as f32;

        if let Square::Occupied(piece, _) = board.get_by_xy(*x, *y) {

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
