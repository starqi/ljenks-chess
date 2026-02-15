// Selects between implementations for random and logging

#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
pub use wasm::*;

#[cfg(not(feature = "wasm"))]
mod cli;
#[cfg(not(feature = "wasm"))]
pub use cli::*;
