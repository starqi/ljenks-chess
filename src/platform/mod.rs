// Platform-specific abstractions

#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
pub use wasm::*;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
pub use cli::*;