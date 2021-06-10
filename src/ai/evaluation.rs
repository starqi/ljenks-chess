use std::collections::HashSet;
use super::super::game::entities::*;
use super::super::game::coords::*;
use super::super::game::board::*;

pub fn evaluate_player(board: &Board, neg_one_if_white_else_one: f32, seven_if_white_else_zero: f32, locs: &HashSet<Coord>) -> f32 {
    let mut value: f32 = 0.;
    for Coord(x, y) in locs.iter() {
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
            value += match piece {
                Piece::Pawn => {
                    seven_if_white_else_zero + neg_one_if_white_else_one * fy * 0.3
                },
                _ => {
                    0.3 * (3.5 - unadvanced)
                }
            };
        }
    }
    value * neg_one_if_white_else_one as f32 * -1.
}

pub fn evaluate(board: &Board) -> f32 {
    evaluate_player(board, 1., 0., &board.get_player_state(Player::Black).piece_locs) +
        evaluate_player(board, -1., 7., &board.get_player_state(Player::White).piece_locs)
}
