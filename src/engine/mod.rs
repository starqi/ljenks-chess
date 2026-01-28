// Shared chess engine code - used by both CLI and WASM
// TODO IMMEDIATE ???

pub mod game;
pub mod ai;

// Re-export commonly used types
pub use ai::*;
pub use game::bitboard_presets::*;
pub use game::memo::*;
pub use game::coords::*;
pub use game::entities::*;
pub use game::board::*;
pub use game::castle_utils::*;
pub use game::searchable_moves::*;
pub use game::move_list::*;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref CASTLE_UTILS: CastleUtils = CastleUtils::new();
    pub static ref RANDOM_NUMBER_KEYS: RandomNumberKeys = RandomNumberKeys::new();
    pub static ref BITBOARD_PRESETS: BitboardPresets = BitboardPresets::new();
}

// Constants
pub const NO_PAWN_HALF_MOVES_DRAW_THRESHOLD: usize = 100;

#[derive(Clone, Debug)]
pub enum GameEndState { 
    WhiteWin = 0, 
    BlackWin = 1, 
    Stalemate = 2, 
    Repetition = 3 
}
