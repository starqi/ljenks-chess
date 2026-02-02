use super::*;

#[derive(Clone)]
pub struct BeforeMoveInfoForStringify {
    pub is_capture: bool,
    pub piece: Option<Piece>,
    pub player_piece_locs: Bitboard,
    pub opponent_piece_locs: Bitboard,
    pub player_same_piece_locs: Option<Bitboard>
}

impl BeforeMoveInfoForStringify {
    /// Must call before the move is made.
    pub fn slow_new(before_board: &Board, m: &MoveWithEval) -> Self {
        let player = before_board.get_player_with_turn();
        let player_state = before_board.get_player_state(player);
        let opponent = player.other_player();
        let opponent_state = before_board.get_player_state(opponent);
        Self {
            is_capture: before_board.is_capture(m),
            piece: match m.description() {
                MoveDescription::NormalMove(from_coord, _, _) => {
                    if let Square::Occupied(p, _) = before_board.get_by_index(from_coord.value()) {
                        Some(*p)
                    } else {
                        None
                    }
                },
                _ => None,
            },
            player_piece_locs: player_state.piece_locs,
            opponent_piece_locs: opponent_state.piece_locs,
            player_same_piece_locs: match m.description() {
                MoveDescription::NormalMove(from_coord, _, _) => {
                    if let Square::Occupied(p, _) = before_board.get_by_index(from_coord.value()) {
                        Some(before_board.slow_create_piece_player_bitboard(player, *p))
                    } else {
                        None
                    }
                },
                _ => None,
            }
        }
    }
}

#[derive(Clone)]
pub struct AfterMoveInfoForStringify {
    pub is_check: bool,
    pub is_checkmate: bool,
}

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
    if count <= 1 { return y_to_rank(y).to_string(); }

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

pub fn stringify_move_for_js_logs(board: &Board, m: &MoveWithEval) -> String {
    match m.description() {
        MoveDescription::NormalMove(_from_coord, _to_coord, metadata) => {
            let square = board.get_by_index(_from_coord.value());
            // Since a piece should be on the after square,
            // the square will stringify to eg. k, K, p, P, then it becomes eg. Ke2
            format!("{}{} ordering={}, metadata={}", square, _to_coord, m.ordering_score(), *metadata as u8)
        },
        MoveDescription::Castle(castle_type) => {
            if *castle_type == CastleType::Oo {
                format!("oo ordering={}", m.ordering_score())
            } else {
                format!("ooo ordering={}", m.ordering_score())
            }
        },
        MoveDescription::SkipMove => {
            format!("skip ordering={}", m.ordering_score())
        }
    }
}

impl Board {
    pub fn print_move_list(&self, ml: &MoveList, start: usize, _end_exclusive: usize) {
        let end_exclusive = if _end_exclusive < ml.v().len() {
            _end_exclusive
        } else {
            ml.v().len()
        };

        console_log!("[Moves, {}-{}]", start, end_exclusive);
        for i in start..end_exclusive {
            console_log!("{}", stringify_move_for_js_logs(self, &ml.v()[i]));
        }
        console_log!("");
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_ambiguous_knight() {
        let mut board = Board::with_kings_only();
        board.set_by_file_rank_test('e', 4, Square::Occupied(Piece::Knight, Player::Black));

        board.set_by_file_rank_test('g', 3, Square::Occupied(Piece::Knight, Player::Black));
        board.set_by_file_rank_test('f', 2, Square::Occupied(Piece::Knight, Player::Black));

        board.set_by_file_rank_test('c', 3, Square::Occupied(Piece::Knight, Player::Black));
        board.set_by_file_rank_test('d', 2, Square::Occupied(Piece::Knight, Player::Black));

        board.set_by_file_rank_test('g', 5, Square::Occupied(Piece::Knight, Player::Black));
        board.set_by_file_rank_test('f', 6, Square::Occupied(Piece::Knight, Player::Black));

        board.set_by_file_rank_test('c', 5, Square::Occupied(Piece::Knight, Player::Black));
        board.set_by_file_rank_test('d', 6, Square::Occupied(Piece::Knight, Player::Black));

        let player_locs = &board.get_player_state(Player::Black).piece_locs;
        let opponent_locs = &board.get_player_state(Player::White).piece_locs;
        let player_knight_locs = &board.slow_create_piece_player_bitboard(Player::Black, Piece::Knight);

        assert_eq!(
            "c3", 
            super::stringify::slow_stringify_unambiguous_coord(
                file_rank_to_xy_safe('c', 3).unwrap(),
                FastCoord::from_coord(&file_rank_to_xy_safe('e', 4).unwrap()),
                Piece::Knight, 
                player_knight_locs,
                player_locs,
                opponent_locs
            )
        );
    }

    #[test]
    fn test_ambiguous_rook() {
        let mut board = Board::with_kings_only();

        board.set_by_file_rank_test('a', 3, Square::Occupied(Piece::Rook, Player::White));
        board.set_by_file_rank_test('a', 4, Square::Occupied(Piece::Rook, Player::White));

        board.set_by_file_rank_test('e', 4, Square::Occupied(Piece::Rook, Player::White));
        board.set_by_file_rank_test('e', 5, Square::Occupied(Piece::Rook, Player::White));

        board.set_by_file_rank_test('e', 1, Square::Occupied(Piece::Rook, Player::White));

        let player_locs = &board.get_player_state(Player::White).piece_locs;
        let opponent_locs = &board.get_player_state(Player::Black).piece_locs;
        let player_rook_locs = &board.slow_create_piece_player_bitboard(Player::White, Piece::Rook);

        assert_eq!(
            "4",
            super::stringify::slow_stringify_unambiguous_coord(
                file_rank_to_xy_safe('e', 4).unwrap(),
                FastCoord::from_coord(&file_rank_to_xy_safe('e', 3).unwrap()),
                Piece::Rook, 
                player_rook_locs,
                player_locs,
                opponent_locs
            )
        );
    }
}
