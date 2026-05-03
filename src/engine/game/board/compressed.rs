use super::*;

pub const COMPRESSED_SIZE: usize = 34;

impl Board {

    /// (Claude probably stole this from somewhere)
    /// 34-byte compressed binary board format:
    /// - Bytes 0..31: 64 squares, 4 bits each (2 squares per byte, lower significance nibble = even index, high nibble = odd index).
    ///   Nibble encoding: 0 = blank, 1..12 = 1 + piece_enum + player_enum * 6.
    /// - Byte 32 flags:
    ///   - Bit 7: side to move (0 = White, 1 = Black)
    ///   - Bit 6: White moved castle piece O-O
    ///   - Bit 5: White moved castle piece O-O-O
    ///   - Bit 4: Black moved castle piece O-O
    ///   - Bit 3: Black moved castle piece O-O-O
    ///   - Bits 2..0: unused
    /// - Byte 33: en passant file + 1 (0 = no ep, 1..8 = file a..h)
    pub fn export_compressed(&self, data: &mut [u8; COMPRESSED_SIZE]) {
        for i in 0..COMPRESSED_SIZE { data[i] = 0; }

        for i in 0..64 {
            let nibble = match self.d[i] {
                Square::Blank => 0u8,
                Square::Occupied(piece, player) => 1 + piece as u8 + player as u8 * 6,
            };
            data[i / 2] |= nibble << ((i & 1) * 4); // First OR with 0 shift, then OR with 4 shift; even, odd, even, odd; big endian like
        }

        let mut flags = 0u8;
        flags |= (self.player_with_turn as u8) << 7;

        let ws = self.get_player_state(Player::White);
        flags |= (ws.moved_castle_piece[CastleType::Oo as usize] as u8) << 6;
        flags |= (ws.moved_castle_piece[CastleType::Ooo as usize] as u8) << 5;

        let bs = self.get_player_state(Player::Black);
        flags |= (bs.moved_castle_piece[CastleType::Oo as usize] as u8) << 4;
        flags |= (bs.moved_castle_piece[CastleType::Ooo as usize] as u8) << 3;

        //println!("CASTLE DEBUG ON EXPORT {:?} {:?}", ws.moved_castle_piece, bs.moved_castle_piece);

        data[32] = flags;
        let ep_code = if self.en_passant_extra_target.has_target() {
            (self.en_passant_extra_target.index % 8) + 1
        } else {
            0
        };
        data[33] = ep_code;
        //println!("FLAGS BYTE ON EXPORT {:?}, EP CODE {:?}", flags, ep_code);
    }

    pub fn import_compressed(&mut self, data: &[u8; COMPRESSED_SIZE]) {
        self.d = [Square::Blank; 64];
        self.player_state = [PlayerState::new(), PlayerState::new()];
        self.en_passant_extra_target.reset();

        for i in 0..64 {
            let nibble = (data[i / 2] >> ((i & 1) * 4)) & 0x0F;
            if nibble > 0 {
                let code = nibble - 1;
                let piece = Piece::from_number(code);
                let player = if code / 6 == 0 { Player::White } else { Player::Black }; // Apparently too annoying to do branchless_mask! for enums here
                self.d[i] = Square::Occupied(piece, player);
                let psm = self.get_player_state_mut(player);
                psm.piece_locs.set_index(i as u8);
                if piece == Piece::King {
                    psm.king_location = Bitboard::from_index(i as u8);
                }
            }
        }

        let flags = data[32];
        let ep_code = data[33];
        //println!("FLAGS BYTE ON IMPORT {:?}, EP CODE {:?}", flags, ep_code);

        self.player_with_turn = if (flags & 0x80) != 0 { Player::Black } else { Player::White };

        let psmw = self.get_player_state_mut(Player::White);
        let psmw_oo = (flags & 0x40) != 0;
        psmw.moved_castle_piece[CastleType::Oo as usize] = psmw_oo;
        let psmw_ooo = (flags & 0x20) != 0;
        psmw.moved_castle_piece[CastleType::Ooo as usize] = psmw_ooo;
        psmw.is_castled = psmw_oo || psmw_ooo;
        //println!("CASTLE DEBUG ON IMPORT WHITE {:?}", psmw.moved_castle_piece);

        let psmb = self.get_player_state_mut(Player::Black); // Fascinating how Rust complains if psmb is placed right below psmw
        let psmb_oo = (flags & 0x10) != 0;
        psmb.moved_castle_piece[CastleType::Oo as usize] = psmb_oo;
        let psmb_ooo = (flags & 0x08) != 0;
        psmb.moved_castle_piece[CastleType::Ooo as usize] = psmb_ooo;
        psmb.is_castled = psmb_oo || psmb_ooo;
        //println!("CASTLE DEBUG ON IMPORT BLACK {:?}", psmb.moved_castle_piece);

        if ep_code > 0 {
            let file = ep_code - 1;
            // Infer y coord:
            // The opponent of the current side-to-move made the double pawn jump.
            // White double-jumps land on internal y=4, ep target y=5.
            // Black double-jumps land on internal y=3, ep target y=2.
            let ep_y = if self.player_with_turn == Player::Black { 5u8 } else { 2u8 };
            self.en_passant_extra_target.set(file, ep_y);
        }

        self.hash = self.calculate_hash();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_boards_equal(a: &Board, b: &Board) {
        assert_eq!(a.player_with_turn, b.player_with_turn, "Side to move mismatch");
        for i in 0..64 {
            assert_eq!(a.d[i], b.d[i], "Square mismatch at index {}", i);
        }
        for player in [Player::White, Player::Black] {
            let sa = a.get_player_state(player);
            let sb = b.get_player_state(player);
            assert_eq!(sa.piece_locs.0, sb.piece_locs.0, "piece_locs mismatch for {:?}", player);
            assert_eq!(sa.king_location.0, sb.king_location.0, "king_location mismatch for {:?}", player);
            assert_eq!(sa.is_castled, sb.is_castled, "is_castled flag mismatch for {:?}", player);
            assert_eq!(sa.moved_castle_piece, sb.moved_castle_piece, "Castle rights mismatch for {:?}", player);
        }
        assert_eq!(a.en_passant_extra_target.has_target(), b.en_passant_extra_target.has_target(), "EP target presence mismatch");
        if a.en_passant_extra_target.has_target() {
            assert_eq!(a.en_passant_extra_target.index, b.en_passant_extra_target.index, "EP target index mismatch");
        }

        // Check hash last to see what failed if any of above imsmatch
        assert_eq!(a.get_hash(), b.get_hash(), "Hash mismatch");
    }

    #[test]
    fn test_roundtrip_starting_position() {
        let board = Board::new();
        let mut data = [0u8; COMPRESSED_SIZE];
        board.export_compressed(&mut data);
        let mut restored = Board::with_kings_only();
        restored.import_compressed(&data);
        assert_boards_equal(&board, &restored);
    }

    #[test]
    fn test_roundtrip_kings_only() {
        let board = Board::with_kings_only();
        let mut data = [0u8; COMPRESSED_SIZE];
        board.export_compressed(&mut data);
        let mut restored = Board::new();
        restored.import_compressed(&data);
        assert_boards_equal(&board, &restored);
    }

    #[test]
    fn test_roundtrip_with_en_passant() {
        let mut board = Board::with_kings_only();
        board.set_by_file_rank_test('e', 2, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('d', 4, Square::Occupied(Piece::Pawn, Player::Black));

        let mut temp = MoveList::new(10);
        let mut result = MoveList::new(10);
        board.get_moves(&mut temp, &mut result);

        let mut found = false;
        for m in result.v() {
            if let MoveDescription::NormalMove(_, _, MoveMetadata::DoublePawnJump) = m.description() {
                let mut revertable = RevertableMove::NoOp(0);
                board.handle_move(m, &mut revertable);
                found = true;
                break;
            }
        }
        assert!(found, "Double pawn jump should exist");
        assert!(board.en_passant_extra_target.has_target());

        let mut data = [0u8; COMPRESSED_SIZE];
        board.export_compressed(&mut data);
        let mut restored = Board::with_kings_only();
        restored.import_compressed(&data);
        assert_boards_equal(&board, &restored);
    }

    #[test]
    fn test_roundtrip_castling_rights_mixed() {
        let mut board = Board::new();
        board.get_player_state_mut(Player::White).moved_castle_piece[CastleType::Oo as usize] = true;
        board.get_player_state_mut(Player::Black).moved_castle_piece[CastleType::Ooo as usize] = true;

        let mut data = [0u8; COMPRESSED_SIZE];
        board.export_compressed(&mut data);
        let mut restored = Board::with_kings_only();
        restored.import_compressed(&data);

        let ws = board.get_player_state(Player::White);
        let ws2 = restored.get_player_state(Player::White);
        assert_eq!(ws.moved_castle_piece, ws2.moved_castle_piece);

        let bs = board.get_player_state(Player::Black);
        let bs2 = restored.get_player_state(Player::Black);
        assert_eq!(bs.moved_castle_piece, bs2.moved_castle_piece);
    }

    #[test]
    fn test_roundtrip_black_to_move() {
        let mut board = Board::new();
        board.handle_move_no_revert(&MoveWithEval(MoveDescription::SkipMove, 0));
        assert_eq!(board.get_player_with_turn(), Player::Black);

        let mut data = [0u8; COMPRESSED_SIZE];
        board.export_compressed(&mut data);
        let mut restored = Board::with_kings_only();
        restored.import_compressed(&data);
        assert_eq!(restored.get_player_with_turn(), Player::Black);
        assert_boards_equal(&board, &restored);
    }
}
