#[cfg(feature = "wasm")]
extern crate console_error_panic_hook;

mod engine;
mod platform;
#[macro_use]
mod macros;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use safetensors::SafeTensors;

pub use engine::game::board::slow_stringify_move_standard;
pub use engine::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Clone, Debug)]
pub enum GameEndState {
    WhiteWin = 0,
    BlackWin = 1,
    Stalemate = 2,
    Repetition = 3
}

impl std::fmt::Display for GameEndState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameEndState::WhiteWin => write!(f, "WhiteWin"),
            GameEndState::BlackWin => write!(f, "BlackWin"),
            GameEndState::Stalemate => write!(f, "Stalemate"),
            GameEndState::Repetition => write!(f, "Repetition"),
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct BestMoveInfoJs {
    pub is_pawn: bool,
    pub remaining_depth: i8,
    pub score: i32,
    notation: String,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl BestMoveInfoJs {
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn notation(&self) -> String { self.notation.clone() }
}

impl From<engine::ai::BestMoveInfo> for BestMoveInfoJs {
    fn from(info: engine::ai::BestMoveInfo) -> Self {
        Self {
            is_pawn: info.is_pawn,
            remaining_depth: info.remaining_depth,
            score: info.objective_score,
            notation: info.notation,
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Main {
    board: Board,
    ai: Ai,

    temp: MoveList,
    move_list: MoveList,
    searchable: SearchableMoves,
    game_end_state: Option<GameEndState>,
    position_hashes: Vec<u64>,
    half_moves_without_pawn_move: usize,
}

/// Loads NNUE safetensors model as a GLOBAL, as if it were baked in to the binary as static data.
/// Note this interfaces with JS, so returns bool for JS with console error logging, not returning `Result`.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn load_weights_safetensors(bytes: &[u8]) -> bool {
    match SafeTensors::deserialize(bytes) {
        Ok(st) => {
            console_log!("Loaded safetensors with {} tensors", st.len());
            let mut input_weight: Option<Box<[f32]>> = None;
            let mut fc1_weight: Option<Box<[f32]>> = None;
            let mut fc1_bias: Option<Box<[f32]>> = None;
            let mut output_weight: Option<Box<[f32]>> = None;
            let mut output_bias: Option<Box<[f32]>> = None;
            for name in st.names() {
                // From safetensors
                // {"fc1.bias":{"dtype":"F32","shape":[32],"data_offsets":[0,128]},
                //  "fc1.weight":{"dtype":"F32","shape":[32,512],"data_offsets":[65664,128]},
                // TODO (Minor) Why do both start with 65664?
                //  "input.weight":{"dtype":"F32","shape":[49162,256],"data_offsets":[65664,50407552]},
                //  "output.bias":{"dtype":"F32","shape":[1],"data_offsets":[50407552,50407556]},
                //  "output.weight":{"dtype":"F32","shape":[1,32],"data_offsets":[50407556,50407684]}}
                match st.tensor(name) {
                    Ok(tensor) => {
                        let shape = tensor.shape();
                        if tensor.dtype() != safetensors::Dtype::F32 {
                            console_log!("Tensor {} has dtype {:?}, expected F32, skipping this tensor", name, tensor.dtype());
                            continue;
                        }
                        // Rust: This is static promotion
                        let expected: &[usize] = match name {
                            "input.weight" => &[Board::NNUE_HALF_SIZE, NNUE_L1_OUTPUT_SIZE],
                            // TODO (Minor) So EmbeddingBag seems to have width/height swapped?
                            "fc1.weight" => &[NNUE_FC1_OUTPUT_SIZE, 2 * NNUE_L1_OUTPUT_SIZE],
                            "fc1.bias" => &[NNUE_FC1_OUTPUT_SIZE],
                            "output.weight" => &[1, NNUE_FC1_OUTPUT_SIZE],
                            "output.bias" => &[1],
                            _ => {
                                console_log!("(Unknown tensor?! {})", name);
                                continue;
                            }
                        };
                        if shape != expected {
                            console_error!("Required tensor {} has shape {:?}, expected {:?}", name, shape, expected);
                            return false;
                        }
                        let data = tensor.data();
                        let f32s: Vec<f32> = data
                            .chunks_exact(4) // This is an iterator but every item is 4 bytes
                            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                            // This illustrates Rust's type craziness:
                            // as long as compiler knows it's a Vec<f32> by explicit typing,
                            // collect() is all about dynamic dispatching to the Vec implementation of Iterator. 
                            .collect();
                        console_log!("  {} {:?} {} values", name, shape, f32s.len());
                        match name {
                            "input.weight" => input_weight = Some(f32s.into_boxed_slice()),
                            "fc1.weight" => fc1_weight = Some(f32s.into_boxed_slice()),
                            "fc1.bias" => fc1_bias = Some(f32s.into_boxed_slice()),
                            "output.weight" => output_weight = Some(f32s.into_boxed_slice()),
                            "output.bias" => output_bias = Some(f32s.into_boxed_slice()),
                            _ => console_log!("(Unknown tensor?! {})", name),
                        }
                    },
                    Err(e) => {
                        console_log!("Failed to load a tensor listed {} {:?}", name, e);
                    }
                }
            }
            match (input_weight, fc1_weight, fc1_bias, output_weight, output_bias) {
                (Some(iw), Some(fw), Some(fb), Some(ow), Some(ob)) => {
                    let w = NnueWeights {
                        input_weight: iw,
                        fc1_weight: fw,
                        fc1_bias: fb,
                        output_weight: ow,
                        output_bias: ob,
                    };
                    match crate::engine::set_nnue_weights(w) {
                        Ok(()) => true,
                        Err(_) => {
                            console_log!("NNUE weights already set, ignoring");
                            true
                        }
                    }
                }
                _ => {
                    console_error!("Missing tensors in safetensors");
                    false
                }
            }
        }
        Err(e) => {
            console_error!("Safetensors deserialize error: {:?}", e);
            false
        }
    }
}

// TODO IMMEDIATE "Main" naming so confusing, main interface???

// This struct does not need to be fast like main engine.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Main {

    // I believe wasm_bindgen(constructor) caused it to require proper JS new keyword instead of function name new
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Main {
        #[cfg(feature = "wasm")]
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
            game_end_state: None,
            position_hashes: position_hashes,
            half_moves_without_pawn_move: 0
        }
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn get_game_end_state(&self) -> Option<GameEndState> {
        self.game_end_state.clone()
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn make_ai_move(&mut self) -> Option<BestMoveInfoJs> {
        self._make_ai_move_with_window(0, 1, false)
    }

    pub fn make_ai_move_with_window(&mut self, center_white_pov: i32, range: i32) -> Option<BestMoveInfoJs> {
        self._make_ai_move_with_window(center_white_pov, range, true)
    }

    pub fn _make_ai_move_with_window(&mut self, center_white_pov: i32, range: i32, use_window: bool) -> Option<BestMoveInfoJs> {
        if self.game_end_state.is_some() {
            console_log!("Game has ended, cannot make AI move");
            return None;
        }

        self.ai.late_inject(&self.position_hashes, &self.half_moves_without_pawn_move);
        let best_move_info = if use_window {
            self.ai.make_move_with_window(&mut self.board, center_white_pov, range)
        } else {
            self.ai.make_move(&mut self.board)
        };
        self.process_best_move_info_to_js(best_move_info)
    }

    /// Not eagerly called during class construction
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn refresh_player_moves(&mut self) {
        self.move_list.write_index = 0;
        self.board.get_moves(&mut self.temp, &mut self.move_list);
        let end_exclusive = self.move_list.write_index;
        console_log!("{} moves", end_exclusive);
        //console_log!("White King\n{}", self.board.get_player_state(Player::White).king_location);
        //console_log!("Black King\n{}", self.board.get_player_state(Player::Black).king_location);
        self.searchable.reset_from_move_list(self.board.get_player_with_turn(), &mut self.move_list, 0, end_exclusive);
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn try_move(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Option<String> {
        if self.game_end_state.is_some() {
            console_log!("Game has ended, cannot make move");
            return None;
        }

        if check_i32_xy(from_x, from_y).is_err() { return None; }
        if check_i32_xy(to_x, to_y).is_err() { return None; }

        let m_clone = self.searchable.get_move(&Coord(from_x as u8, from_y as u8), &Coord(to_x as u8, to_y as u8)).map(|x| x.clone());
        if let Some(ref m) = m_clone {
            let before_info = BeforeMoveInfoForStringify::slow_new(&self.board, m);
            let original_player = self.board.get_player_with_turn();
            let is_pawn = matches!(self.board.get_moved_piece(m), Some(Piece::Pawn));

            self.board.handle_move_no_revert(m);
            self.board.assert_hash();

            let mut is_checkmate = false;
            let mut is_stalemate = false;
            let mut is_check = false;
            self.handle_special_end_conditions(original_player, is_pawn, &mut is_checkmate, &mut is_stalemate, &mut is_check);
            let after_info = AfterMoveInfoForStringify { is_check, is_checkmate };
            Some(slow_stringify_move_standard(m, &before_info, &after_info))
        } else {
            None
        }
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn get_piece(&self, x: i32, y: i32) -> i8 {
        if let Ok(Square::Occupied(piece, player)) = self.board.get_by_xy_safe(x, y) {
            ((*piece as u8) + 1) as i8 * player.multiplier() as i8
        } else if let Ok(Square::Blank) = self.board.get_by_xy_safe(x, y) {
            0
        } else {
            -99
        }
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn evaluate(&mut self) -> Option<BestMoveInfoJs> {
        self._evaluate_with_window(0, 1, false)
    }

    pub fn evaluate_with_window(&mut self, center_white_pov: i32, range: i32) -> Option<BestMoveInfoJs> {
        self._evaluate_with_window(center_white_pov, range, true)
    }

    fn _evaluate_with_window(&mut self, center_white_pov: i32, range: i32, use_window: bool) -> Option<BestMoveInfoJs> {
        if self.game_end_state.is_some() {
            return None;
        }
        self.ai.late_inject(&self.position_hashes, &self.half_moves_without_pawn_move);
        if use_window {
            self.ai.evaluate_with_window(&self.board, center_white_pov, range).map(BestMoveInfoJs::from)
        } else {
            self.ai.evaluate(&self.board).map(BestMoveInfoJs::from)
        }
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn get_player_with_turn(&self) -> u8 {
        self.board.get_player_with_turn() as u8
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn new_board(&mut self) {
        self.board = Board::new();
        let initial_hash = self.board.get_hash();
        self.position_hashes.clear();
        self.position_hashes.push(initial_hash);
        // Sanity check: Inner Ai class doesn't need reset,
        // many objects like move list, move buckets will completely rewrite themselves.
        // Memo re-used. Late-injected stuff also fine. 
        // Similarly, this class's temp, move_list, searchable are also reset on every move.
        self.half_moves_without_pawn_move = 0;
        self.game_end_state = None;
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn make_random_move(&mut self) -> Option<BestMoveInfoJs> {
        if self.game_end_state.is_some() {
            console_log!("Game has ended, cannot make random move");
            return None;
        }

        self.ai.late_inject(&self.position_hashes, &self.half_moves_without_pawn_move);
        let best_move_info = self.ai.make_random_move(&mut self.board);
        self.process_best_move_info_to_js(best_move_info)
    }

    #[cfg(not(feature = "wasm"))]
    pub fn set_search_max_nodes(&mut self, max_nodes: Option<u64>) {
        self.ai.set_search_max_nodes(max_nodes);
    }

    #[cfg(not(feature = "wasm"))]
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    #[cfg(not(feature = "wasm"))]
    pub fn get_node_counter(&self) -> u64 {
        self.ai.get_node_counter()
    }

    #[cfg(not(feature = "wasm"))]
    pub fn set_logging(&self, enabled: bool) {
        platform::set_logging(enabled);
    }

    /// Returns white-perspective (objective) score.
    #[cfg(not(feature = "wasm"))]
    pub fn static_evaluate_objective(&self) -> i32 {
        ai::evaluate(&self.board)
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub fn load_fen(&mut self, fen: &str) -> bool {
        match Board::from_fen(fen) {
            Ok(new_board) => {
                self.board = new_board;
                self.position_hashes = vec![self.board.get_hash()];
                self.half_moves_without_pawn_move = 0;
                self.game_end_state = None;
                self.refresh_player_moves();
                true
            },
            Err(_) => false,
        }
    }

    fn process_best_move_info_to_js(&mut self, best_move_info: Option<BestMoveInfo>) -> Option<BestMoveInfoJs> {
        let is_pawn_holder = best_move_info.as_ref().map(|x| x.is_pawn); // Just need is_pawn here
        if let Some(is_pawn) = is_pawn_holder {
            self.handle_special_end_conditions(
                self.board.get_player_with_turn().other_player(),
                is_pawn, &mut false, &mut false, &mut false,
            );
            best_move_info.map(BestMoveInfoJs::from)
        } else {
            console_error!("No moves, but game has not ended?!");
            None
        }
    }

    /// Handles precise game end conditions.
    /// Board struct will only be in terms of # of moves unavailable or whether is checking,
    /// but this excludes formal checkmate, stalemate, 50 move rule, etc.
    fn handle_special_end_conditions(
        &mut self,
        original_player: Player,
        is_last_move_pawn: bool,

        // Output immediate vars to help caller
        is_checkmate_output: &mut bool,
        is_stalemate_output: &mut bool,
        is_check_output: &mut bool
    ) {
        *is_check_output = self.board.is_checking(original_player);
        let has_no_legal_moves = self.board.has_no_legal_moves();
        *is_checkmate_output = *is_check_output && has_no_legal_moves;
        *is_stalemate_output = !*is_check_output && has_no_legal_moves;

        let current_hash = self.board.get_hash();
        self.position_hashes.push(current_hash);

        if is_last_move_pawn {
            self.half_moves_without_pawn_move = 1;
        } else {
            self.half_moves_without_pawn_move += 1;
        }
        console_log!("half_moves_without_pawn_move {}", self.half_moves_without_pawn_move);

        self.game_end_state = if *is_checkmate_output {
            if original_player == Player::White {
                Some(GameEndState::WhiteWin)
            } else {
                Some(GameEndState::BlackWin)
            }
        } else if *is_stalemate_output {
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
        let r = main.try_move(4, 6, 4, 4);
        assert_eq!(r.unwrap_or_default(), "e4");

        // Reset for next test
        let mut main = Main::new();
        main.refresh_player_moves();

        // Test knight move g1-f3
        let r2 = main.try_move(6, 7, 5, 5); // g1 to f3
        assert_eq!(r2.unwrap_or_default(), "Nf3");
    }
}
