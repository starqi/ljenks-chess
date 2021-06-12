mod extern_funcs;
mod macros;
mod game;
mod ai;

use ai::{Ai};
use game::board::*;
use game::castle_utils::*;
use game::entities::*;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Main {
    board: Board,
    castle_utils: CastleUtils,
    ai: Ai
}

#[wasm_bindgen]
impl Main {

    pub fn new() -> Main {
        Main {
            board: Board::new(),
            castle_utils: CastleUtils::new(),
            ai: Ai::new()
        }
    }

    pub fn make_move(&mut self) {
        self.ai.make_move(&self.castle_utils, 5, &mut self.board);
    }

    pub fn get_piece(&self, x: i32, y: i32) -> i8 {
        if let Ok(Square::Occupied(piece, player)) = self.board.get_by_xy_safe(x, y) {
            let piece2: u8 = (piece as u8) + 1;
            let player2: i8 = if player == Player::Black { -1 } else { 1 };
            (piece2 as i8) * player2
        } else if let Ok(Square::Blank) = self.board.get_by_xy_safe(x, y) {
            0
        } else {
            -99
        }
    }
}
