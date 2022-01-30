use super::move_list::*;
use super::entities::*;
use super::coords::*;

/// Size 2 arrays are indexed by `Player` enum numbers
pub struct CastleUtils {
    pub oo_sqs: [[BeforeAfterSquare; 4]; 2],
    pub ooo_sqs: [[BeforeAfterSquare; 5]; 2],
    pub oo_king_traversal_coords: [[Coord; 2]; 2],
    pub ooo_king_traversal_coords: [[Coord; 3]; 2],
    pub oo_draggable_coords: [[(Coord, Coord); 3]; 2],
    pub ooo_draggable_coords: [[(Coord, Coord); 3]; 2]
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
        crate::console_log!("Generating castle constants");

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
            oo_king_traversal_coords: [
                [Coord(6, white_first_row), Coord(5, white_first_row)],
                [Coord(6, black_first_row), Coord(5, black_first_row)]
            ],
            ooo_king_traversal_coords: [
                [Coord(1, white_first_row), Coord(2, white_first_row), Coord(3, white_first_row)],
                [Coord(1, black_first_row), Coord(2, black_first_row), Coord(3, black_first_row)]
            ],
            oo_draggable_coords: [[
                (Coord(4, 7), Coord(7, 7)),
                (Coord(7, 7), Coord(4, 7)),
                (Coord(4, 7), Coord(6, 7))
            ], [
                (Coord(4, 0), Coord(7, 0)),
                (Coord(7, 0), Coord(4, 0)),
                (Coord(4, 0), Coord(6, 0))
            ]],
            ooo_draggable_coords: [[
                (Coord(0, 7), Coord(4, 7)),
                (Coord(4, 7), Coord(0, 7)),
                (Coord(4, 7), Coord(3, 7))
            ], [
                (Coord(0, 0), Coord(4, 0)),
                (Coord(4, 0), Coord(0, 0)),
                (Coord(4, 0), Coord(3, 0))
            ]]
        };
    }
}
