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

use std::sync::LazyLock;

pub static CASTLE_UTILS: LazyLock<CastleUtils> = LazyLock::new(|| CastleUtils::new());
pub static RANDOM_NUMBER_KEYS: LazyLock<RandomNumberKeys> = LazyLock::new(|| RandomNumberKeys::new());
pub static BITBOARD_PRESETS: LazyLock<BitboardPresets> = LazyLock::new(|| BitboardPresets::new());

pub fn init_globals() {
    LazyLock::force(&CASTLE_UTILS);
    LazyLock::force(&RANDOM_NUMBER_KEYS);
    LazyLock::force(&BITBOARD_PRESETS);
}

pub fn castle_utils() -> &'static CastleUtils {
    &CASTLE_UTILS
}

pub fn random_number_keys() -> &'static RandomNumberKeys {
    &RANDOM_NUMBER_KEYS
}

pub fn bitboard_presets() -> &'static BitboardPresets {
    &BITBOARD_PRESETS
}

pub const NO_PAWN_HALF_MOVES_DRAW_THRESHOLD: usize = 100;
