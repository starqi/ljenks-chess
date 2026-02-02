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

use std::sync::OnceLock;

pub static CASTLE_UTILS: OnceLock<CastleUtils> = OnceLock::new();
pub static RANDOM_NUMBER_KEYS: OnceLock<RandomNumberKeys> = OnceLock::new();
pub static BITBOARD_PRESETS: OnceLock<BitboardPresets> = OnceLock::new();

pub fn init_globals() {
    CASTLE_UTILS.get_or_init(|| CastleUtils::new());
    RANDOM_NUMBER_KEYS.get_or_init(|| RandomNumberKeys::new());
    BITBOARD_PRESETS.get_or_init(|| BitboardPresets::new());
}

pub fn castle_utils() -> &'static CastleUtils {
    CASTLE_UTILS.get().expect("CASTLE_UTILS not initialized")
}

pub fn random_number_keys() -> &'static RandomNumberKeys {
    RANDOM_NUMBER_KEYS.get().expect("RANDOM_NUMBER_KEYS not initialized")
}

pub fn bitboard_presets() -> &'static BitboardPresets {
    BITBOARD_PRESETS.get().expect("BITBOARD_PRESETS not initialized")
}

pub const NO_PAWN_HALF_MOVES_DRAW_THRESHOLD: usize = 100;
