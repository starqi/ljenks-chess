use crate::game::{
    bitboard::Bitboard,
    board::{AfterMoveInfoForStringify, BeforeMoveInfoForStringify},
    coords::{x_to_file, xy_to_file_rank, y_to_rank, Coord, FastCoord},
    entities::Piece,
    move_list::{CastleType, MoveDescription, MoveWithEval}, 
    move_test::{_write_bishop_moves_self_capture, _write_knight_moves_self_capture, _write_queen_moves_self_capture, _write_rook_moves_self_capture}
};

// TODO IMMEDIATE Nc5xe4 is a thing right?
/// e.g. For Nxe4, fills in Nc5xe4, or the c part of Ra1 -> Rac1.
/// Does not handle pawns, do that elsewhere.
/// Does not handle castle or special moves, that's why `piece` is required.
/// King move will always return empty string.
/// Note this does not anything in the post-move "after" board state. 
pub fn slow_stringify_unambiguous_coord(
    from_coord: Coord,
    to_coord: FastCoord,
    piece: Piece,
    player_before_same_pieces: &Bitboard,
    player_before_pieces: &Bitboard,
    opponent_before_pieces: &Bitboard,
) -> String {
    let reverse_move_gen = match piece { // Because chess movements are "commutative"
        Piece::King => return String::new(),
        Piece::Queen => _write_queen_moves_self_capture(to_coord, player_before_pieces, opponent_before_pieces),
        Piece::Knight => _write_knight_moves_self_capture(to_coord),
        Piece::Bishop => _write_bishop_moves_self_capture(to_coord, player_before_pieces, opponent_before_pieces),
        Piece::Rook => _write_rook_moves_self_capture(to_coord, player_before_pieces, opponent_before_pieces),
        _ => {
            panic!("Cannot call this method with piece {}", piece);
        }
    };
    //println!("{}", reverse_move_gen);
    //println!("{}", player_before_same_pieces);
    let candidates_bb = Bitboard(reverse_move_gen.0 & player_before_same_pieces.0);
    //println!("{}", candidates_bb);
    if candidates_bb.0 == 0 {
        panic!("Could not find piece being moved during move stringify, piece = {}", piece);
    }
    if candidates_bb.pop_count() == 1 { // The piece name ("R", "rook") is enough to identify location
        return String::new();
    }


    let x = from_coord.0;
    let y = from_coord.1;

    let mut count = 0;
    for j in 0..8 {
        if candidates_bb.is_set(x, j) { count += 1; }
    }
    //println!("Count along y {}", count);
    if count <= 1 { return x_to_file(x).to_string(); }

    count = 0;
    for i in 0..8 {
        if candidates_bb.is_set(i, y) { count += 1; }
    }
    //println!("Count along x {}", count);
    if count <= 1 { return y_to_rank(x).to_string(); }

    format!("{}{}", x_to_file(x), y_to_rank(y))
}

pub fn slow_stringify_move_standard(
    m: &MoveWithEval,
    before: &BeforeMoveInfoForStringify,
    after: &AfterMoveInfoForStringify
) -> String {
    let mut result = match m.description() {
        MoveDescription::NormalMove(_from_coord, _to_coord, _metadata) => {
            let to_coord = _to_coord.to_coord();
            let piece = before.piece.expect("Illegal state: Found normal move without a piece during stringify");

            let piece_str = match piece {
                Piece::Pawn => "",
                Piece::Knight => "N",
                Piece::Bishop => "B",
                Piece::Rook => "R",
                Piece::Queen => "Q",
                Piece::King => "K",
            };

            let (to_file, to_rank) = xy_to_file_rank(to_coord.0, to_coord.1);
            let to_str = format!("{}{}", to_file, to_rank);

            let from_coord = _from_coord.to_coord();
            let (from_file, _) = xy_to_file_rank(from_coord.0, from_coord.1);
            if piece == Piece::Pawn {
                if before.is_capture {
                    format!("{}x{}", from_file, to_str)
                } else {
                    to_str
                }
            } else {
                const EXPECT_MSG1: &str = "Expected player_same_piece_locs to be populated in this scenario.";
                if before.is_capture {
                    format!("{}{}x{}", 
                        piece_str, 
                        slow_stringify_unambiguous_coord(
                            from_coord,
                            *_to_coord,
                            piece,
                            &before.player_same_piece_locs.expect(EXPECT_MSG1),
                            &before.player_piece_locs,
                            &before.opponent_piece_locs
                        ),
                        to_str
                    )
                } else {
                    format!("{}{}{}", 
                        piece_str,
                        slow_stringify_unambiguous_coord(
                            from_coord,
                            *_to_coord,
                            piece,
                            &before.player_same_piece_locs.expect(EXPECT_MSG1),
                            &before.player_piece_locs,
                            &before.opponent_piece_locs
                        ),
                        to_str
                    )
                }
            }
        },
        MoveDescription::Castle(castle_type) => {
            if *castle_type == CastleType::Oo {
                "O-O".to_string()
            } else {
                "O-O-O".to_string()
            }
        },
        MoveDescription::SkipMove => {
            "<Skip>".to_string()
        }
    };

    if after.is_checkmate {
        result.push('#');
    } else if after.is_check {
        result.push('+');
    }

    result
}
