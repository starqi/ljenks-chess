use super::move_list::*;
use super::entities::*;
use super::coords::*;
use super::super::*;

/// Size 2 arrays are indexed by `Player` enum numbers.
/// When not split by oo/ooo, first index: `CastleType` enum number, then `Player` enum number
pub struct CastleUtils {
    pub oo_sqs: [[BeforeAfterSquare; 4]; 2],
    pub ooo_sqs: [[BeforeAfterSquare; 5]; 2],

    pub oo_blank_coords: [[Coord; 2]; 2],
    pub ooo_blank_coords: [[Coord; 3]; 2],

    pub king_traversal_coords: [[[Coord; 2]; 2]; 2],
    pub draggable_coords: [[[(Coord, Coord); 3]; 2]; 2],
}

impl CastleUtils {

    fn oo_squares_for_row(player: Player) -> [BeforeAfterSquare; 4] {
        let y = player.first_row();
        [
            BeforeAfterSquare(FastCoord::from_xy(4, y), Square::Occupied(Piece::King, player), Square::Blank),
            BeforeAfterSquare(FastCoord::from_xy(5, y), Square::Blank, Square::Occupied(Piece::Rook, player)),
            BeforeAfterSquare(FastCoord::from_xy(6, y), Square::Blank, Square::Occupied(Piece::King, player)),
            BeforeAfterSquare(FastCoord::from_xy(7, y), Square::Occupied(Piece::Rook, player), Square::Blank)
        ]
    }

    fn ooo_squares_for_row(player: Player) -> [BeforeAfterSquare; 5] {
        let y = player.first_row();
        [
            BeforeAfterSquare(FastCoord::from_xy(0, y), Square::Occupied(Piece::Rook, player), Square::Blank),
            BeforeAfterSquare(FastCoord::from_xy(1, y), Square::Blank, Square::Blank),
            BeforeAfterSquare(FastCoord::from_xy(2, y), Square::Blank, Square::Occupied(Piece::King, player)),
            BeforeAfterSquare(FastCoord::from_xy(3, y), Square::Blank, Square::Occupied(Piece::Rook, player)),
            BeforeAfterSquare(FastCoord::from_xy(4, y), Square::Occupied(Piece::King, player), Square::Blank)
        ]
    }

    pub fn new() -> CastleUtils {
        console_log!("Generating castle constants");

        let white_first_row = Player::first_row(Player::White);
        let black_first_row = Player::first_row(Player::Black);

        return CastleUtils {
            oo_sqs: [
                CastleUtils::oo_squares_for_row(Player::White),
                CastleUtils::oo_squares_for_row(Player::Black)
            ],
            ooo_sqs: [
                CastleUtils::ooo_squares_for_row(Player::White),
                CastleUtils::ooo_squares_for_row(Player::Black)
            ],
            king_traversal_coords: [[
                [Coord(6, white_first_row), Coord(5, white_first_row)],
                [Coord(6, black_first_row), Coord(5, black_first_row)]
            ], [
                [Coord(2, white_first_row), Coord(3, white_first_row)],
                [Coord(2, black_first_row), Coord(3, black_first_row)]
            ]],
            oo_blank_coords: [
                [Coord(6, white_first_row), Coord(5, white_first_row)],
                [Coord(6, black_first_row), Coord(5, black_first_row)]
            ],
            ooo_blank_coords: [
                [Coord(1, white_first_row), Coord(2, white_first_row), Coord(3, white_first_row)],
                [Coord(1, black_first_row), Coord(2, black_first_row), Coord(3, black_first_row)]
            ],
            draggable_coords: [[[
                (Coord(4, 7), Coord(7, 7)),
                (Coord(7, 7), Coord(4, 7)),
                (Coord(4, 7), Coord(6, 7))
            ], [
                (Coord(4, 0), Coord(7, 0)),
                (Coord(7, 0), Coord(4, 0)),
                (Coord(4, 0), Coord(6, 0))
            ]], [[
                (Coord(0, 7), Coord(4, 7)),
                (Coord(4, 7), Coord(0, 7)),
                (Coord(4, 7), Coord(2, 7))
            ], [
                (Coord(0, 0), Coord(4, 0)),
                (Coord(4, 0), Coord(0, 0)),
                (Coord(4, 0), Coord(2, 0))
            ]]]
        };
    }
}
