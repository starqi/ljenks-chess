extern crate console_error_panic_hook;

mod engine;
mod platform; 
#[macro_use]
mod macros;

use wasm_bindgen::prelude::*;

use engine::*;
use crate::engine::game::board::slow_stringify_move_standard;

// WASM bindgen recommendation.
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Clone)]
pub enum GameEndState { 
    WhiteWin = 0, 
    BlackWin = 1, 
    Stalemate = 2, 
    Repetition = 3 
}

#[wasm_bindgen]
pub struct Main {
    board: Board,
    ai: Ai,

    temp: MoveList,
    move_list: MoveList,
    searchable: SearchableMoves,
    last_move: Option<String>,
    game_end_state: Option<GameEndState>,
    position_hashes: Vec<u64>,
    half_moves_without_pawn_move: usize,
    last_ai_evaluation: Option<i32>
}

#[wasm_bindgen]
impl Main {

    // I believe wasm_bindgen(constructor) caused it to require proper JS new keyword instead of function name new
    #[wasm_bindgen(constructor)]
    pub fn new() -> Main {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Initialize globals explicitly
        init_globals();

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
            position_hashes: position_hashes,
            half_moves_without_pawn_move: 0,
            last_ai_evaluation: None
        }
    }

    #[wasm_bindgen]
    pub fn get_game_end_state(&self) -> Option<GameEndState> {
        self.game_end_state.clone()
    }

    #[wasm_bindgen]
    pub fn make_ai_move(&mut self) -> bool {
        if self.game_end_state.is_some() {
            console_log!("Game has ended, cannot make AI move");
            return false;
        }

        self.ai.late_inject(&self.position_hashes, &self.half_moves_without_pawn_move);
        self.last_move = self.ai.make_move(&mut self.board);
        self.last_ai_evaluation = self.ai.get_leading_move_with_score().map(|(_move, _depth, score)| score);

        self.slow_handle_special_end_conditions(self.board.get_player_with_turn().other_player(), None);
        true
    }

    #[wasm_bindgen]
    pub fn refresh_player_moves(&mut self) {
        self.move_list.write_index = 0;
        self.board.get_moves(&mut self.temp, &mut self.move_list);
        let end_exclusive = self.move_list.write_index;
        console_log!("{} moves", end_exclusive);
        //console_log!("White King\n{}", self.board.get_player_state(Player::White).king_location);
        //console_log!("Black King\n{}", self.board.get_player_state(Player::Black).king_location);
        self.searchable.reset_from_move_list(self.board.get_player_with_turn(), &mut self.move_list, 0, end_exclusive);
    }

    #[wasm_bindgen]
    pub fn try_move(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool {
        if self.game_end_state.is_some() {
            console_log!("Game has ended, cannot make AI move");
            return false;
        }

        if check_i32_xy(from_x, from_y).is_err() { return false; }
        if check_i32_xy(to_x, to_y).is_err() { return false; }

        let _m = self.searchable.get_move(&Coord(from_x as u8, from_y as u8), &Coord(to_x as u8, to_y as u8));
        if let Some(m) = _m {
            let m_clone = m.clone(); // Dodge borrow checker
            let before_info = BeforeMoveInfoForStringify::slow_new(&self.board, m);
            let original_player = self.board.get_player_with_turn();

            self.board.handle_move_no_revert(m);
            self.board.assert_hash();

            self.slow_handle_special_end_conditions(original_player, Some((&before_info, &m_clone)));

            true
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub fn get_piece(&self, x: i32, y: i32) -> i8 {
        if let Ok(Square::Occupied(piece, player)) = self.board.get_by_xy_safe(x, y) {
            ((*piece as u8) + 1) as i8 * player.multiplier() as i8
        } else if let Ok(Square::Blank) = self.board.get_by_xy_safe(x, y) {
            0
        } else {
            -99
        }
    }

    #[wasm_bindgen]
    pub fn get_last_move_notation(&self) -> String {
        self.last_move.clone().unwrap_or_default()
    }

    #[wasm_bindgen]
    pub fn get_last_ai_evaluation(&self) -> Option<i32> {
        self.last_ai_evaluation
    }

    #[wasm_bindgen]
    pub fn get_player_with_turn(&self) -> u8 {
        self.board.get_player_with_turn() as u8
    }

    #[wasm_bindgen]
    pub fn load_fen(&mut self, fen: &str) -> bool {
        match Board::from_fen(fen) {
            Ok(new_board) => {
                self.board = new_board;
                self.position_hashes = vec![self.board.get_hash()];
                self.half_moves_without_pawn_move = 0;
                self.last_move = None;
                // TODO IMMEDIATE Edge cases: checkmate, stalemate, no kings
                self.game_end_state = None;
                self.last_ai_evaluation = None;
                self.refresh_player_moves();
                true
            },
            Err(_) => false,
        }
    }

    // TODO IMMEDIATE Does this belong here?
    // Board class will only be in terms of # of moves unavailable or whether is checking,
    // but this excludes formal checkmate, stalemate, 50 move rule, etc.
    fn slow_handle_special_end_conditions(
        &mut self,
        original_player: Player,
        before_info_if_setting_last_move: Option<(&BeforeMoveInfoForStringify, &MoveWithEval)>
    ) {
        let is_check = self.board.is_checking(original_player);
        let has_no_legal_moves = self.board.has_no_legal_moves();
        let is_checkmate = is_check && has_no_legal_moves;
        let is_stalemate = !is_check && has_no_legal_moves;

        if let Some((before, m)) = before_info_if_setting_last_move {
            let after_info = AfterMoveInfoForStringify { is_check, is_checkmate };
            let notation = slow_stringify_move_standard(m, before, &after_info);
            self.last_move = Some(notation);
        }

        let current_hash = self.board.get_hash();
        self.position_hashes.push(current_hash);

        if let Some(m) = &self.last_move {
            if m.len() == 2 { // TODO IMMEDIATE Proper pawn detection, repeated code?
                self.half_moves_without_pawn_move = 1;
            } else {
                self.half_moves_without_pawn_move += 1;
            }
            console_log!("half_moves_without_pawn_move {} {}", self.half_moves_without_pawn_move, m);
        }

        self.game_end_state = if is_checkmate {
            if original_player == Player::White {
                Some(GameEndState::WhiteWin)
            } else {
                Some(GameEndState::BlackWin)
            }
        } else if is_stalemate {
            Some(GameEndState::Stalemate)
        } else {
            let repetition_count = self.position_hashes.iter().filter(|&&h| h == current_hash).count();
            if repetition_count >= 3 {
                console_log!("Repetition called {}", repetition_count);
                Some(GameEndState::Repetition)
            } else {
                if self.half_moves_without_pawn_move >= NO_PAWN_HALF_MOVES_DRAW_THRESHOLD {
                    console_log!("Repetition called no pawn moves {}", self.half_moves_without_pawn_move);
                    Some(GameEndState::Repetition)
                } else {
                    None
                }
            }
        };
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
