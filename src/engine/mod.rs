pub mod ai;
pub mod game;

// Re-export commonly used types
pub use ai::*;
pub use game::bitboard_presets::*;
pub use game::board::*;
pub use game::castle_utils::*;
pub use game::coords::*;
pub use game::entities::*;
pub use game::memo::*;
pub use game::move_list::*;
pub use game::searchable_moves::*;

use std::sync::LazyLock;
use std::sync::OnceLock;

pub static CASTLE_UTILS: LazyLock<CastleUtils> = LazyLock::new(|| CastleUtils::new());
pub static RANDOM_NUMBER_KEYS: LazyLock<RandomNumberKeys> = LazyLock::new(|| RandomNumberKeys::new());
pub static BITBOARD_PRESETS: LazyLock<BitboardPresets> = LazyLock::new(|| BitboardPresets::new());

pub struct NnueWeights {
    pub input_weight: Box<[f32]>,
    pub fc1_weight: Box<[f32]>,
    pub fc1_bias: Box<[f32]>,
    pub fc2_weight: Box<[f32]>,
    pub fc2_bias: Box<[f32]>,
    pub output_weight: Box<[f32]>,
    pub output_bias: Box<[f32]>,
}

pub static NNUE_WEIGHTS: OnceLock<NnueWeights> = OnceLock::new();

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

pub fn nnue_input_weights() -> Option<&'static [f32]> {
    NNUE_WEIGHTS.get().map(|w| &(*w.input_weight))
}

pub fn nnue_fc1_weights() -> Option<&'static [f32]> {
    NNUE_WEIGHTS.get().map(|w| &(*w.fc1_weight))
}

pub fn nnue_fc1_biases() -> Option<&'static [f32]> {
    NNUE_WEIGHTS.get().map(|w| &(*w.fc1_bias))
}

pub fn nnue_fc2_weights() -> Option<&'static [f32]> {
    NNUE_WEIGHTS.get().map(|w| &(*w.fc2_weight))
}

pub fn nnue_fc2_biases() -> Option<&'static [f32]> {
    NNUE_WEIGHTS.get().map(|w| &(*w.fc2_bias))
}

pub fn nnue_output_weights() -> Option<&'static [f32]> {
    NNUE_WEIGHTS.get().map(|w| &(*w.output_weight))
}

pub fn nnue_output_biases() -> Option<&'static [f32]> {
    NNUE_WEIGHTS.get().map(|w| &(*w.output_bias))
}

pub fn set_nnue_weights(weights: NnueWeights) -> Result<(), NnueWeights> {
    NNUE_WEIGHTS.set(weights)
}

pub const NO_PAWN_HALF_MOVES_DRAW_THRESHOLD: usize = 100;
