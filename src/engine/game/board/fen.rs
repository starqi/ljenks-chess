// (CORRECTNESS IS AI REVIEWED) //

use super::*;

#[derive(Debug, PartialEq)]
pub enum FenError {
    InvalidFormat,
    InvalidPiecePlacement,
    InvalidActiveColor,
    InvalidCastlingRights,
    InvalidEnPassantSquare,
    // TODO (Minor) Not supported
    //InvalidHalfmoveClock,
    //InvalidFullmoveNumber,
}

impl Display for FenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            FenError::InvalidFormat => write!(f, "Invalid FEN format"),
            FenError::InvalidPiecePlacement => write!(f, "Invalid piece placement"),
            FenError::InvalidActiveColor => write!(f, "Invalid active color"),
            FenError::InvalidCastlingRights => write!(f, "Invalid castling rights"),
            FenError::InvalidEnPassantSquare => write!(f, "Invalid en passant square"),
            // TODO (Minor) Not supported
            //FenError::InvalidHalfmoveClock => write!(f, "Invalid halfmove clock"),
            //FenError::InvalidFullmoveNumber => write!(f, "Invalid fullmove number"),
        }
    }
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut board = Self {
            d: [Square::Blank; 64],
            hash: 0,
            player_with_turn: Player::White,
            player_state: [PlayerState::new(), PlayerState::new()],
            en_passant_extra_target: TargetSquare::new(),
            nnue_acc: [[0.0; NNUE_L1_OUTPUT_SIZE]; 2],
        };

        board.load_fen(fen)?;
        Ok(board)
    }

    /// No half-move and full move counter implemented at the moment.
    pub fn load_fen(&mut self, fen: &str) -> Result<(), FenError> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() != 6 {
            return Err(FenError::InvalidFormat);
        }

        self.parse_set_piece_placement(parts[0])?;

        // Parse active color
        self.player_with_turn = match parts[1] {
            "w" => Player::White,
            "b" => Player::Black,
            _ => return Err(FenError::InvalidActiveColor),
        };

        self.parse_set_castling_rights(parts[2])?;
        self.parse_set_en_passant(parts[3])?;

        self.update_derived_state();
        self.hash = self.calculate_hash();

        self.nnue_refresh_both(); // TODO IMMEDIATE Remove comment when FEN load is tested for NNUE
        Ok(())
    }

    fn parse_set_piece_placement(&mut self, placement: &str) -> Result<(), FenError> {
        let ranks: Vec<&str> = placement.split('/').collect();
        if ranks.len() != 8 {
            return Err(FenError::InvalidPiecePlacement);
        }

        for (rank_idx, rank) in ranks.iter().enumerate() {
            let mut file_idx = 0;

            for ch in rank.chars() {
                if file_idx >= 8 {
                    return Err(FenError::InvalidPiecePlacement);
                }

                if ch.is_digit(10) {
                    let empty_count = ch.to_digit(10).unwrap() as usize;
                    if file_idx + empty_count > 8 {
                        return Err(FenError::InvalidPiecePlacement);
                    }
                    for _ in 0..empty_count {
                        if file_idx < 8 {
                            let coord = FastCoord::from_xy(file_idx as u8, rank_idx as u8);
                            self.d[coord.value() as usize] = Square::Blank;
                            file_idx += 1;
                        }
                    }
                } else {
                    let (piece, player) = Self::parse_fen_piece(ch)?;
                    let coord = FastCoord::from_xy(file_idx as u8, rank_idx as u8);
                    self.d[coord.value() as usize] = Square::Occupied(piece, player);
                    file_idx += 1;
                }
            }

            if file_idx != 8 {
                return Err(FenError::InvalidPiecePlacement);
            }
        }

        Ok(())
    }

    fn parse_fen_piece(ch: char) -> Result<(Piece, Player), FenError> {
        match ch {
            'P' => Ok((Piece::Pawn, Player::White)),
            'N' => Ok((Piece::Knight, Player::White)),
            'B' => Ok((Piece::Bishop, Player::White)),
            'R' => Ok((Piece::Rook, Player::White)),
            'Q' => Ok((Piece::Queen, Player::White)),
            'K' => Ok((Piece::King, Player::White)),
            'p' => Ok((Piece::Pawn, Player::Black)),
            'n' => Ok((Piece::Knight, Player::Black)),
            'b' => Ok((Piece::Bishop, Player::Black)),
            'r' => Ok((Piece::Rook, Player::Black)),
            'q' => Ok((Piece::Queen, Player::Black)),
            'k' => Ok((Piece::King, Player::Black)),
            _ => Err(FenError::InvalidPiecePlacement),
        }
    }

    fn parse_set_castling_rights(&mut self, castling: &str) -> Result<(), FenError> {
        for player in [Player::White, Player::Black] {
            self.get_player_state_mut(player).moved_castle_piece = [true, true];
        }

        if castling == "-" {
            return Ok(());
        }

        for ch in castling.chars() {
            match ch {
                'K' => self.get_player_state_mut(Player::White).moved_castle_piece[0] = false,
                'Q' => self.get_player_state_mut(Player::White).moved_castle_piece[1] = false,
                'k' => self.get_player_state_mut(Player::Black).moved_castle_piece[0] = false,
                'q' => self.get_player_state_mut(Player::Black).moved_castle_piece[1] = false,
                _ => return Err(FenError::InvalidCastlingRights),
            }
        }

        Ok(())
    }

    fn parse_set_en_passant(&mut self, en_passant: &str) -> Result<(), FenError> {
        if en_passant == "-" {
            self.en_passant_extra_target.reset();
            return Ok(());
        }

        if en_passant.len() != 2 {
            return Err(FenError::InvalidEnPassantSquare);
        }

        let file = en_passant.chars().next().unwrap();
        let rank = en_passant.chars().nth(1).unwrap();

        if !('a' <= file && file <= 'h') || !('1' <= rank && rank <= '8') {
            return Err(FenError::InvalidEnPassantSquare);
        }

        let x = file as u8 - b'a';
        let y = rank as u8 - b'1';

        // En passant is only valid on ranks 3 and 6
        if y != 2 && y != 5 {
            return Err(FenError::InvalidEnPassantSquare);
        }

        self.en_passant_extra_target.set(x, 7 - y);
        Ok(())
    }

    fn update_derived_state(&mut self) {
        let mut white_piece_locs = Bitboard(0);
        let mut black_piece_locs = Bitboard(0);
        let mut white_king_loc = Bitboard(0);
        let mut black_king_loc = Bitboard(0);

        for (idx, square) in self.d.iter().enumerate() {
            if let Square::Occupied(piece, player) = square {
                let bitboard = Bitboard::from_index(idx as u8);

                match player {
                    Player::White => {
                        white_piece_locs = white_piece_locs | bitboard;
                        if *piece == Piece::King {
                            white_king_loc = bitboard;
                        }
                    }
                    Player::Black => {
                        black_piece_locs = black_piece_locs | bitboard;
                        if *piece == Piece::King {
                            black_king_loc = bitboard;
                        }
                    }
                }
            }
        }

        self.get_player_state_mut(Player::White).piece_locs = white_piece_locs;
        self.get_player_state_mut(Player::White).king_location = white_king_loc;
        self.get_player_state_mut(Player::Black).piece_locs = black_piece_locs;
        self.get_player_state_mut(Player::Black).king_location = black_king_loc;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fen_from_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();

        // Check that pieces are in starting positions
        assert_eq!(*board.get_by_file_rank_safe('a', 1).unwrap(), Square::Occupied(Piece::Rook, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('b', 1).unwrap(), Square::Occupied(Piece::Knight, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('c', 1).unwrap(), Square::Occupied(Piece::Bishop, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('d', 1).unwrap(), Square::Occupied(Piece::Queen, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('e', 1).unwrap(), Square::Occupied(Piece::King, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('f', 1).unwrap(), Square::Occupied(Piece::Bishop, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('g', 1).unwrap(), Square::Occupied(Piece::Knight, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('h', 1).unwrap(), Square::Occupied(Piece::Rook, Player::White));

        assert_eq!(*board.get_by_file_rank_safe('a', 2).unwrap(), Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('b', 2).unwrap(), Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('c', 2).unwrap(), Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('d', 2).unwrap(), Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('e', 2).unwrap(), Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('f', 2).unwrap(), Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('g', 2).unwrap(), Square::Occupied(Piece::Pawn, Player::White));
        assert_eq!(*board.get_by_file_rank_safe('h', 2).unwrap(), Square::Occupied(Piece::Pawn, Player::White));

        assert_eq!(*board.get_by_file_rank_safe('a', 8).unwrap(), Square::Occupied(Piece::Rook, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('b', 8).unwrap(), Square::Occupied(Piece::Knight, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('c', 8).unwrap(), Square::Occupied(Piece::Bishop, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('d', 8).unwrap(), Square::Occupied(Piece::Queen, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('e', 8).unwrap(), Square::Occupied(Piece::King, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('f', 8).unwrap(), Square::Occupied(Piece::Bishop, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('g', 8).unwrap(), Square::Occupied(Piece::Knight, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('h', 8).unwrap(), Square::Occupied(Piece::Rook, Player::Black));

        assert_eq!(*board.get_by_file_rank_safe('a', 7).unwrap(), Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('b', 7).unwrap(), Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('c', 7).unwrap(), Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('d', 7).unwrap(), Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('e', 7).unwrap(), Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('f', 7).unwrap(), Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('g', 7).unwrap(), Square::Occupied(Piece::Pawn, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('h', 7).unwrap(), Square::Occupied(Piece::Pawn, Player::Black));

        // Check that center squares are empty
        assert_eq!(*board.get_by_file_rank_safe('d', 4).unwrap(), Square::Blank);
        assert_eq!(*board.get_by_file_rank_safe('e', 4).unwrap(), Square::Blank);
        assert_eq!(*board.get_by_file_rank_safe('d', 5).unwrap(), Square::Blank);
        assert_eq!(*board.get_by_file_rank_safe('e', 5).unwrap(), Square::Blank);

        // Check turn and castling rights
        assert_eq!(board.get_player_with_turn(), Player::White);
        assert_eq!(board.get_player_state(Player::White).moved_castle_piece, [false, false]);
        assert_eq!(board.get_player_state(Player::Black).moved_castle_piece, [false, false]);
    }

    #[test]
    fn test_fen_black_to_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(board.get_player_with_turn(), Player::Black);
    }

    #[test]
    fn test_fen_castling_rights() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kk - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(
            board.get_player_state(Player::White).moved_castle_piece,
            [false, true]
        );
        assert_eq!(
            board.get_player_state(Player::Black).moved_castle_piece,
            [false, true]
        );

        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Qk - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(
            board.get_player_state(Player::White).moved_castle_piece,
            [true, false]
        );
        assert_eq!(
            board.get_player_state(Player::Black).moved_castle_piece,
            [false, true]
        );

        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(
            board.get_player_state(Player::White).moved_castle_piece,
            [true, true]
        );
        assert_eq!(
            board.get_player_state(Player::Black).moved_castle_piece,
            [true, true]
        );
    }

    #[test]
    fn test_fen_en_passant() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR w KQkq e6 0 2";
        let board = Board::from_fen(fen).unwrap();
        assert!(board.en_passant_extra_target.has_target());
        assert_eq!(board.en_passant_extra_target.index, FastCoord::from_xy(4, 2).0); // e6 (file e=4, rank 6 -> y=2)

        // No en passant target
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert!(!board.en_passant_extra_target.has_target());
    }

    #[test]
    fn test_fen_custom_position() {
        // A simple position with just a few pieces
        let fen = "8/8/8/4k3/8/3K4/8/8 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();

        // Check king positions
        assert_eq!(*board.get_by_file_rank_safe('e', 5).unwrap(), Square::Occupied(Piece::King, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('d', 3).unwrap(), Square::Occupied(Piece::King, Player::White));

        // Check all other squares are blank
        for file in b'a'..=b'h' {
            for rank in 1..=8 {
                let file_char = file as char;
                if (file_char, rank) != ('e', 5) && (file_char, rank) != ('d', 3) {
                    assert_eq!(*board.get_by_file_rank_safe(file_char, rank).unwrap(), Square::Blank);
                }
            }
        }
    }

    #[test]
    fn test_fen_invalid_format() {
        assert!(Board::from_fen("").is_err());
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").is_err()); // missing parts
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -").is_err()); // missing one part
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra").is_err()); // extra parts
    }

    #[test]
    fn test_fen_invalid_piece_placement() {
        assert!(Board::from_fen("rnbqkbn/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err()); // missing piece
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err()); // too many pieces
        assert!(Board::from_fen("rnbqkbnr/pppppppp/9/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err()); // invalid number
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8x8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err()); // invalid character
    }

    #[test]
    fn test_fen_invalid_active_color() {
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1").is_err()); // invalid color
    }

    #[test]
    fn test_fen_invalid_castling_rights() {
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkx - 0 1").is_err()); // invalid castling
    }

    #[test]
    fn test_fen_invalid_en_passant() {
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq i5 0 1").is_err()); // invalid file
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e9 0 1").is_err()); // invalid rank
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e55 0 1").is_err()); // invalid format
    }

    #[test]
    fn test_fen_load_into_existing_board() {
        let mut board = Board::new();
        let fen = "8/8/8/4k3/8/3K4/8/8 w - - 0 1";

        assert!(board.load_fen(fen).is_ok());

        assert_eq!(*board.get_by_file_rank_safe('e', 5).unwrap(), Square::Occupied(Piece::King, Player::Black));
        assert_eq!(*board.get_by_file_rank_safe('d', 3).unwrap(), Square::Occupied(Piece::King, Player::White));
        assert_eq!(board.get_player_with_turn(), Player::White);
    }
}
