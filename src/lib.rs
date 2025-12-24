#[macro_use]
extern crate lazy_static;
extern crate console_error_panic_hook;

#[cfg(test)]
extern crate rand;

mod extern_funcs;
mod macros;
mod game;
mod ai;

use ai::*;
use game::bitboard_presets::*;
use game::memo::*;
use game::coords::*;
use game::entities::*;
use game::board::*;
use game::castle_utils::*;
use game::searchable_moves::*;
use game::move_list::*;
use wasm_bindgen::prelude::*;

use crate::game::stringify::slow_stringify_move_standard;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    pub static ref CASTLE_UTILS: CastleUtils = CastleUtils::new();
    pub static ref RANDOM_NUMBER_KEYS: RandomNumberKeys = RandomNumberKeys::new();
    pub static ref BITBOARD_PRESETS: BitboardPresets = BitboardPresets::new();
}

#[wasm_bindgen]
#[derive(Clone)]
pub enum GameEndState { Checkmate = 0, Stalemate = 1, RepetitionCalled = 2 }

#[wasm_bindgen]
pub struct Main {
    board: Board,
    ai: Ai,

    temp: MoveList,
    move_list: MoveList,
    searchable: SearchableMoves,
    last_move: Option<String>,
    game_end_state: Option<GameEndState>,
    position_hashes: Vec<u64>
}

#[wasm_bindgen]
impl Main {

    pub fn new() -> Main {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Initialize lazy
        let _ = &CASTLE_UTILS.oo_sqs;
        let _ = &CASTLE_UTILS.ooo_sqs;
        let _ = &CASTLE_UTILS.king_traversal_coords;
        let _ = &CASTLE_UTILS.draggable_coords;
        let _ = &RANDOM_NUMBER_KEYS.squares;
        let _ = &BITBOARD_PRESETS.knight_jumps;
        let _ = &BITBOARD_PRESETS.rays;

        let board = Board::new();
        let initial_hash = board.get_hash();
        let position_hashes = vec![initial_hash];
        Main {
            board,
            ai: Ai::new(),

            temp: MoveList::new(50),
            move_list: MoveList::new(50),
            searchable: SearchableMoves::new(),
            last_move: None,
            game_end_state: None,
            position_hashes: position_hashes
        }
    }

    pub fn get_game_end_state(&self) -> Option<GameEndState> {
        self.game_end_state.clone()
    }

    pub fn make_ai_move(&mut self) -> bool {
        if self.game_end_state.is_some() {
            console_log!("Game has ended, cannot make AI move");
            return false;
        }

        self.ai.late_inject(&self.position_hashes);
        self.last_move = self.ai.make_move(&mut self.board);

        let is_check = self.board.is_checking(self.board.get_player_with_turn().other_player());
        let has_no_legal_moves = self.board.has_no_legal_moves();
        let is_checkmate = is_check && has_no_legal_moves;
        let is_stalemate = !is_check && has_no_legal_moves;

        let current_hash = self.board.get_hash();
        self.position_hashes.push(current_hash);
        let repetition_count = self.position_hashes.iter().filter(|&&h| h == current_hash).count();
        self.game_end_state = if is_checkmate {
            Some(GameEndState::Checkmate)
        } else if is_stalemate {
            Some(GameEndState::Stalemate)
        } else if repetition_count >= 3 {
            console_log!("DRAW WHAT {}", repetition_count);
            Some(GameEndState::RepetitionCalled)
        } else {
            None
        };
        true
    }

    pub fn refresh_player_moves(&mut self) {
        self.move_list.write_index = 0;
        self.board.get_moves(&mut self.temp, &mut self.move_list);
        let end_exclusive = self.move_list.write_index;
        console_log!("{} moves", end_exclusive);
        //console_log!("White King\n{}", self.board.get_player_state(Player::White).king_location);
        //console_log!("Black King\n{}", self.board.get_player_state(Player::Black).king_location);
        self.searchable.reset_from_move_list(self.board.get_player_with_turn(), &mut self.move_list, 0, end_exclusive);
    }

    pub fn try_move(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool {
        if self.game_end_state.is_some() {
            console_log!("Game has ended, cannot make AI move");
            return false;
        }

        if check_i32_xy(from_x, from_y).is_err() { return false; }
        if check_i32_xy(to_x, to_y).is_err() { return false; }

        let _m = self.searchable.get_move(&Coord(from_x as u8, from_y as u8), &Coord(to_x as u8, to_y as u8));
        if let Some(m) = _m {
            let before_info = BeforeMoveInfoForStringify::slow_new(&self.board, m);
            let original_player = self.board.get_player_with_turn();

            self.board.handle_move_no_revert(m);
            self.board.assert_hash();

            let is_check = self.board.is_checking(original_player);
            let has_no_legal_moves = self.board.has_no_legal_moves();
            let is_checkmate = is_check && has_no_legal_moves;
            let is_stalemate = !is_check && has_no_legal_moves;
            let after_info = AfterMoveInfoForStringify { is_check, is_checkmate };

            let notation = slow_stringify_move_standard(m, &before_info, &after_info);
            self.last_move = Some(notation);

            let current_hash = self.board.get_hash();

            self.position_hashes.push(current_hash);
            let repetition_count = self.position_hashes.iter().filter(|&&h| h == current_hash).count();
            self.game_end_state = if is_checkmate {
                Some(GameEndState::Checkmate)
            } else if is_stalemate {
                Some(GameEndState::Stalemate)
            } else if repetition_count >= 3 {
                Some(GameEndState::RepetitionCalled)
            } else {
                None
            };
            true
        } else {
            false
        }
    }

    pub fn get_piece(&self, x: i32, y: i32) -> i8 {
        if let Ok(Square::Occupied(piece, player)) = self.board.get_by_xy_safe(x, y) {
            ((*piece as u8) + 1) as i8 * player.multiplier() as i8
        } else if let Ok(Square::Blank) = self.board.get_by_xy_safe(x, y) {
            0
        } else {
            -99
        }
    }

    pub fn get_last_move_notation(&self) -> String {
        self.last_move.clone().unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_notation_basic_moves() {
        let mut main = Main::new();
        main.refresh_player_moves();

        // Test pawn move e2-e4
        assert!(main.try_move(4, 6, 4, 4)); // e2 to e4
        assert_eq!(main.get_last_move_notation(), "e4");

        // Reset for next test
        let mut main = Main::new();
        main.refresh_player_moves();

        // Test knight move g1-f3
        assert!(main.try_move(6, 7, 5, 5)); // g1 to f3
        assert_eq!(main.get_last_move_notation(), "Nf3");
    }
}
