use std::fmt::{self, Display, Formatter};

use crate::*;

use super::bitboard::*;
use super::coords::*;
use super::entities::*;
use super::memo::*;
use super::move_test::*;

// Child modules
mod compressed;
mod fen;
mod moves;
mod nnue;
mod stringify;

// Public re-exports
pub use compressed::*;
pub use fen::*;
pub use moves::*;
pub use nnue::*;
pub use stringify::*;

#[derive(Clone)]
pub struct PlayerState {
    pub piece_locs: Bitboard,
    pub king_location: Bitboard,
    pub is_castled: bool,
    /// Index: `CastleType` enum number
    pub moved_castle_piece: [bool; 2]
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            piece_locs: Bitboard(0),
            king_location: Bitboard(0),
            moved_castle_piece: [false, false],
            is_castled: false
        }
    }
}

#[derive(Clone)]
pub struct TargetSquare {
    bitboard: Bitboard,
    index: u8
}

impl TargetSquare {
    pub fn new() -> Self {
        TargetSquare { bitboard: Bitboard(0), index: 0 }
    }

    pub fn has_target(&self) -> bool {
        self.bitboard.0 != 0
    }

    pub fn reset(&mut self) {
        self.bitboard.0 = 0;
        self.index = 0;
    }

    pub fn set(&mut self, x: u8, y: u8) {
        self.index = FastCoord::from_xy(x, y).0;
        self.bitboard = Bitboard::from_index(self.index);
    }
}

pub const NNUE_L1_OUTPUT_SIZE: usize = 256;

#[derive(Clone)]
pub struct Board {
    player_with_turn: Player,
    d: [Square; 64],
    hash: u64,
    player_state: [PlayerState; 2],
    en_passant_extra_target: TargetSquare,
    nnue_acc: [[f32; NNUE_L1_OUTPUT_SIZE]; 2], // Indexed by `Player` enum
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        for i in 0..self.d.len() {
            if i % 8 == 0 && i != 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}", self.d[i])?;
        }
        Ok(())
    }
}

// TODO (Feature Req) Outside ability to set the board to mostly anything without breaking hash,
// right now tests have responsibilty to maintain proper state

/// Will not track full game end conditions such as stalemate, or 50 move rule, or repetitions,
/// will only know whether there exist moves or not (e.g. none during checkmate). Caller must handle this.
impl Board {

    /// Sets up a standard board
    pub fn new() -> Self {
        let mut board = Self {
            d: [Square::Blank; 64],
            hash: 0,
            player_with_turn: Player::White,
            player_state: [PlayerState::new(), PlayerState::new()],
            en_passant_extra_target: TargetSquare::new(),
            nnue_acc: [[0.0; NNUE_L1_OUTPUT_SIZE]; 2],
        };
        board.set_standard_rows();
        board.get_player_state_mut(Player::White).king_location =
            Bitboard::from_index(castle_utils().pre_castle_king_sq[Player::White as usize].0);
        board.get_player_state_mut(Player::Black).king_location =
            Bitboard::from_index(castle_utils().pre_castle_king_sq[Player::Black as usize].0);
        board.hash = board.calculate_hash();
        board.nnue_refresh(Player::White);
        board.nnue_refresh(Player::Black);
        board
    }

    /// Creates a board with only kings on their starting squares
    pub fn with_kings_only() -> Self {
        let mut board = Self {
            d: [Square::Blank; 64],
            hash: 0,
            player_with_turn: Player::White,
            player_state: [PlayerState::new(), PlayerState::new()],
            en_passant_extra_target: TargetSquare::new(),
            nnue_acc: [[0.0; NNUE_L1_OUTPUT_SIZE]; 2],
        };

        board.set_by_file_rank('e', 1, Square::Occupied(Piece::King, Player::White));
        board.set_by_file_rank('e', 8, Square::Occupied(Piece::King, Player::Black));
        board.get_player_state_mut(Player::White).king_location =
            Bitboard::from_index(FastCoord::from_xy(4, 7).0);
        board.get_player_state_mut(Player::Black).king_location =
            Bitboard::from_index(FastCoord::from_xy(4, 0).0);
        board.get_player_state_mut(Player::White).moved_castle_piece = [false, false];
        board.get_player_state_mut(Player::Black).moved_castle_piece = [false, false];
        board.hash = board.calculate_hash();
        board.nnue_refresh(Player::White);
        board.nnue_refresh(Player::Black);
        board
    }

    //////////////////////////////////////////////////
    // Misc

    pub fn slow_create_piece_player_bitboard(&self, player: Player, piece: Piece) -> Bitboard {
        let mut bb = Bitboard(0);
        self.get_player_state(player).piece_locs.clone().consume_loop_indices(|index| {
            if let Square::Occupied(curr_piece, curr_player) = self.get_by_index(index) {
                if *curr_piece == piece && *curr_player == player {
                    bb.set_index(index);
                }
            }
        });
        bb
    }

    //////////////////////////////////////////////////
    // Hashes

    /// Slow hash calculation from scratch, currently just for assertions
    pub fn calculate_hash(&self) -> u64 {
        let mut h: u64 = 0;

        let mut i = 0usize;
        for sq in self.d.iter() {
            if let Square::Occupied(piece, player) = sq {
                h ^= Self::get_square_hash(i, *piece, *player);
            }
            i += 1;
        }

        let ws = self.get_player_state(Player::White);
        let bs = self.get_player_state(Player::Black);

        if ws.moved_castle_piece[CastleType::Oo as usize] {
            h ^= random_number_keys().moved_castle_piece[CastleType::Oo as usize][Player::White as usize]; 
        }
        if ws.moved_castle_piece[CastleType::Ooo as usize] {
            h ^= random_number_keys().moved_castle_piece[CastleType::Ooo as usize][Player::White as usize]; 
        }

        if bs.moved_castle_piece[CastleType::Oo as usize] {
            h ^= random_number_keys().moved_castle_piece[CastleType::Oo as usize][Player::Black as usize]; 
        }
        if bs.moved_castle_piece[CastleType::Ooo as usize] {
            h ^= random_number_keys().moved_castle_piece[CastleType::Ooo as usize][Player::Black as usize]; 
        } 

        if self.get_player_with_turn() == Player::White { h ^= random_number_keys().is_white_to_play; }
        
        if self.en_passant_extra_target.has_target() {
            h ^= Self::get_loc_hash(self.en_passant_extra_target.index);
        }
        
        h
    }

    #[inline]
    pub fn get_hash(&self) -> u64 {
        self.hash
    }

    pub fn assert_hash(&self) {
        assert_eq!(self.hash, self.calculate_hash());
    }

    #[inline]
    fn get_loc_hash_en_passant(en_passant_extra_target: &TargetSquare) -> u64 {
        Self::get_loc_hash(en_passant_extra_target.index)
    }

    #[inline]
    fn get_square_hash(i: usize, piece: Piece, player: Player) -> u64 {
        random_number_keys().squares[i * PER_SQUARE_LEN + (piece as usize) + (player as usize) * PIECE_LEN]
    }

    #[inline]
    fn get_loc_hash(index: u8) -> u64 {
        random_number_keys().locations[index as usize]
    }

    //////////////////////////////////////////////////
    // Player state

    #[inline]
    pub fn get_player_with_turn(&self) -> Player {
        self.player_with_turn
    }

    #[inline]
    pub fn get_player_state(&self, player: Player) -> &PlayerState {
        &self.player_state[player as usize]
    }

    #[inline]
    pub fn get_player_state_mut(&mut self, player: Player) -> &mut PlayerState {
        &mut self.player_state[player as usize]
    }

    //////////////////////////////////////////////////
    // Get set squares

    pub fn get_by_file_rank_safe(&self, file: char, rank: u8) -> Result<&Square, Error> {
        let Coord(x, y) = file_rank_to_xy_safe(file, rank)?;
        Ok(self.get_by_xy(x, y))
    }

    pub fn get_by_xy_safe(&self, x: i32, y: i32) -> Result<&Square, Error> {
        check_i32_xy(x, y)?;
        Ok(self.get_by_xy(x as u8, y as u8))
    }

    #[inline]
    pub fn get_by_xy(&self, x: u8, y: u8) -> &Square {
        &self.d[y as usize * 8 + x as usize]
    }

    #[inline]
    pub fn get_by_index(&self, num: u8) -> &Square {
        &self.d[num as usize]
    }

    #[inline]
    pub fn get_by_fast_coord(&self, fc: FastCoord) -> &Square {
        self.get_by_index(fc.0)
    }

    #[cfg(test)]
    pub fn set_by_file_rank_test(&mut self, file: char, rank: u8, s: Square) {
        self.set_by_file_rank(file, rank, s);
    }

    fn set_by_file_rank(&mut self, file: char, rank: u8, s: Square) {
        let Coord(x, y) = file_rank_to_xy(file, rank);
        self.set_by_xy(x, y, s);
    }

    #[inline]
    fn set_by_xy(&mut self, x: u8, y: u8, s: Square) {
        self.set_by_index(y * 8 + x, s);
    }

    fn set_by_index_no_hash(&mut self, index: u8, s: Square) {
        self.get_player_state_mut(Player::White).piece_locs.unset_index(index);
        self.get_player_state_mut(Player::Black).piece_locs.unset_index(index);
        if let Square::Occupied(_, new_player) = s {
            self.get_player_state_mut(new_player).piece_locs.set_index(index);
        }
        self.d[index as usize] = s;
    }

    fn set_by_index(&mut self, index: u8, s: Square) {
        self.get_player_state_mut(Player::White).piece_locs.unset_index(index);
        self.get_player_state_mut(Player::Black).piece_locs.unset_index(index);

        if let Square::Occupied(replaced_piece, replaced_piece_player) = self.get_by_index(index) {
            self.hash ^= Self::get_square_hash(index as usize, *replaced_piece, *replaced_piece_player);
        }
        if let Square::Occupied(new_piece, new_player) = s {
            self.hash ^= Self::get_square_hash(index as usize, new_piece, new_player);
            self.get_player_state_mut(new_player).piece_locs.set_index(index);
        }

        self.d[index as usize] = s;
    }

    //////////////////////////////////////////////////
    // Board setup

    fn set_uniform_row(&mut self, rank: u8, sq: Square) {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank, sq);
        }
    }

    #[cfg(test)]
    pub fn set_uniform_row_test(&mut self, rank: u8, sq: Square) {
        self.set_uniform_row(rank, sq);
    }

    fn set_main_row(&mut self, rank: u8, player: Player) {
        self.set_by_file_rank('a', rank, Square::Occupied(Piece::Rook, player));
        self.set_by_file_rank('b', rank, Square::Occupied(Piece::Knight, player));
        self.set_by_file_rank('c', rank, Square::Occupied(Piece::Bishop, player));
        self.set_by_file_rank('d', rank, Square::Occupied(Piece::Queen, player));
        self.set_by_file_rank('e', rank, Square::Occupied(Piece::King, player));
        self.set_by_file_rank('f', rank, Square::Occupied(Piece::Bishop, player));
        self.set_by_file_rank('g', rank, Square::Occupied(Piece::Knight, player));
        self.set_by_file_rank('h', rank, Square::Occupied(Piece::Rook, player));
    }

    fn set_standard_rows(&mut self) {
        self.set_main_row(1, Player::White);
        self.set_uniform_row(2, Square::Occupied(Piece:: Pawn, Player::White));
        self.set_main_row(8, Player::Black);
        self.set_uniform_row(7, Square::Occupied(Piece::Pawn, Player::Black));
    }

    pub fn has_no_legal_moves(&mut self) -> bool {
        let mut temp_moves = MoveList::new(50);
        let mut result_moves = MoveList::new(50);
        self.get_moves(&mut temp_moves, &mut result_moves);
        result_moves.write_index == 0
    }

    pub fn stringify_move_for_js_logs(&self, m: &MoveWithEval) -> String {
        stringify::stringify_move_for_js_logs(self, m)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[ignore]
    #[test]
    fn board_eyeball_test() {
        let mut board = Board::new();
        board.set_uniform_row(2, Square::Blank);
        board.set_uniform_row(7, Square::Blank);

        let mut ml = MoveList::new(100);
        board._get_pseudo_moves_at(FastCoord::from_xy(0, 0), Player::White, &mut ml);

        let mut b = Bitboard(0);
        for m in ml.v() {
            if let MoveDescription::NormalMove(_, _to, _) = m.description() {
                b.set_index(_to.0);
            }
        }
        println!("{}", b);
    }

    #[ignore]
    #[test]
    fn attacked_from_eyeball_test() {
        let mut board = Board::new();
        board.set_uniform_row(2, Square::Blank);
        board.set_uniform_row(7, Square::Blank);

        let mut af = AttackFromBoards::new();
        board.rewrite_af_boards_both_players(&mut af);
        for y in 0..8 {
            for x in 0..8 {
                println!("{},{}\n{}", x, y, af.data[y * 8 + x]);
            }
        }
    }

    #[ignore]
    #[test]
    fn cc_eyeball_test() {
        let mut board = Board::new();
        board.set_uniform_row(2, Square::Blank);
        board.set_uniform_row(7, Square::Blank);
        board.set_by_file_rank_test('a', 2, Square::Occupied(Piece::Pawn, Player::Black));
        board.set_by_file_rank_test('d', 3, Square::Occupied(Piece::Pawn, Player::Black));
        board.set_by_file_rank_test('f', 3, Square::Occupied(Piece::Pawn, Player::Black));
        board.set_by_file_rank_test('e', 3, Square::Occupied(Piece::Pawn, Player::Black));

        let mut temp = MoveList::new(100);
        let mut result = MoveList::new(100);
        board.get_checks_captures_for(Player::Black, &mut temp, &mut result);

        for m in result.v() {
            if let MoveDescription::NormalMove(_from, _to, _) = m.description() {
                let mut b = Bitboard(0);
                b.set_index(_from.0);
                b.set_index(_to.0);
                println!("{}", b);
            }
        }
    }

    #[ignore]
    #[test]
    fn cc_eyeball_test2() {
        let mut board = Board::with_kings_only();
        // FIXME IMMEDIATE ???
        board.set_by_file_rank_test('d', 2, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('e', 5, Square::Occupied(Piece::King, Player::Black));
        board.set_by_file_rank_test('a', 1, Square::Occupied(Piece::King, Player::White));
        board.get_player_state_mut(Player::White).king_location =
            Bitboard::from_index(FastCoord::from_xy(0, 7).0);
        board.get_player_state_mut(Player::Black).king_location =
            Bitboard::from_index(FastCoord::from_xy(4, 3).0);

        let mut temp = MoveList::new(10);
        let mut result = MoveList::new(10);
        board.get_checks_captures_for(Player::White, &mut temp, &mut result);

        for m in result.v() {
            if let MoveDescription::NormalMove(_from, _to, _) = m.description() {
                let mut b = Bitboard(0);
                b.set_index(_from.0);
                b.set_index(_to.0);
                println!("{}", b);
            }
        }
    }

    #[test]
    fn test_en_passant() {
        let mut board = Board::with_kings_only();
        assert_eq!(Player::White, board.get_player_with_turn(), "White should be default starting player");
        
        board.set_by_file_rank_test('e', 2, Square::Occupied(Piece::Pawn, Player::White));
        board.set_by_file_rank_test('d', 4, Square::Occupied(Piece::Pawn, Player::Black));
        
        // Generate moves for white and find the double jump move

        let mut temp = MoveList::new(10);
        let mut result = MoveList::new(10);
        board.get_moves(&mut temp, &mut result);

        let mut double_jump_move = None;
        for m in result.v() {
            if let MoveDescription::NormalMove(from, to, metadata) = m.description() {
                if *metadata == MoveMetadata::DoublePawnJump {
                    let from_coord = from.to_coord();
                    let to_coord = to.to_coord();
                    // Should be e2 to e4
                    if from_coord.0 == 4 && from_coord.1 == 6 && to_coord.0 == 4 && to_coord.1 == 4 {
                        double_jump_move = Some(m.clone());
                        break;
                    }
                }
            }
        }
        assert!(double_jump_move.is_some(), "Double pawn jump move should be available");
        
        // Execute the double jump move
        assert!(board.en_passant_extra_target.bitboard.0 == 0, "En passant target is clear before double jump");

        let mut revertable = RevertableMove::NoOp(0);
        board.handle_move(&double_jump_move.unwrap(), &mut revertable);
        assert!(board.en_passant_extra_target.bitboard.0 != 0, "En passant target is set after double jump");

        // Now generate moves for black and find the en passant move

        let mut temp = MoveList::new(10);
        let mut result = MoveList::new(10);
        board.get_moves(&mut temp, &mut result);

        let mut en_passant_move = None;
        for m in result.v() {
            if let MoveDescription::NormalMove(from, to, metadata) = m.description() {
                if *metadata == MoveMetadata::EnPassant {
                    let from_coord = from.to_coord();
                    let to_coord = to.to_coord();
                    // Should be d4 to e3
                    if from_coord.0 == 3 && from_coord.1 == 4 && to_coord.0 == 4 && to_coord.1 == 5 {
                        en_passant_move = Some(m.clone());
                        break;
                    }
                }
            }
        }
        assert!(en_passant_move.is_some(), "En passant move should be available");
        
        board.handle_move_no_revert(&en_passant_move.unwrap());
        assert!(matches!(board.get_by_file_rank_safe('d', 4), Ok(Square::Blank)), "Captured pawn at d4 should be gone");
        assert!(matches!(board.get_by_file_rank_safe('e', 3), Ok(Square::Occupied(Piece::Pawn, Player::Black))), "Black pawn should be at e3");
        assert!(matches!(board.get_by_file_rank_safe('e', 4), Ok(Square::Blank)), "White pawn at e4 should be gone");
        assert!(board.en_passant_extra_target.bitboard.0 == 0, "En passant target is cleared after en passant move");
    }
}
