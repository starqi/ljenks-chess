#[macro_use]
extern crate lazy_static;

mod extern_funcs;
mod macros;
mod game;
mod ai;

use ai::{Ai};
use game::board::*;
use game::castle_utils::*;
use game::entities::*;
use game::searchable_moves::*;
use game::move_list::*;
use game::coords::*;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    pub static ref CASTLE_UTILS: CastleUtils = CastleUtils::new();
}

#[wasm_bindgen]
pub struct Main {
    board: Board,
    ai: Ai,

    temp: MoveList,
    move_list: MoveList,
    searchable: SearchableMoves
}

#[wasm_bindgen]
impl Main {

    pub fn new() -> Main {
        let board = Board::new();
        Main {
            board, 
            ai: Ai::new(),

            temp: MoveList::new(50),
            move_list: MoveList::new(50),
            searchable: SearchableMoves::new()
        }
    }

    pub fn make_ai_move(&mut self) {
        self.ai.make_move(5, &mut self.board);
    }

    pub fn refresh_player_moves(&mut self) {
        self.move_list.write_index = 0;
        self.board.get_moves(&mut self.temp, &mut self.move_list);
        self.searchable.reset(&mut self.move_list);
    }

    pub fn try_move(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool {
        if check_i32_xy(from_x, from_y).is_err() { return false; }
        if check_i32_xy(to_x, to_y).is_err() { return false; }

        let _m = self.searchable.get_move(Coord(from_x as u8, from_y as u8), Coord(to_x as u8, to_y as u8));
        if let Some(m) = _m {
            self.board.make_move(m);
            true
        } else {
            false
        }
    }

    pub fn get_piece(&self, x: i32, y: i32) -> i8 {
        if let Ok(Square::Occupied(piece, player)) = self.board.get_by_xy_safe(x, y) {
            ((piece as u8) + 1) as i8 * player.get_multiplier() as i8
        } else if let Ok(Square::Blank) = self.board.get_by_xy_safe(x, y) {
            0
        } else {
            -99
        }
    }
}
