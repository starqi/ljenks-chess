mod board;
mod ai;

use ai::{Ai};
use board::{MoveList, CastleUtils, Board, Square, Player};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
struct Main {
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

        /*

           let counter_ref = Arc::clone(&ai.counter);
           thread::spawn(move || {
           let duration = std::time::Duration::from_secs(5);
           loop {
           {
           let counter = counter_ref.lock().unwrap();
           println!("Evaluations = {}", counter);
           }
           thread::sleep(duration);
           }
           });

           let mut temp_moves = MoveList::new(10);
           let mut moves_result = MoveList::new(10);


            loop {
                io::stdin().read_line(&mut y).expect("?");
        */

        self.ai.make_move(&self.castle_utils, 3, &mut self.board);
    }

    pub fn get_piece(&self, x: u8, y: u8) -> i8 {
        if let Ok(Square::Occupied(piece, player)) = self.board.get_by_xy(x, y) {
            let piece2: u8 = (piece as u8) + 1;
            let player2: i8 = if player == Player::Black { -1 } else { 1 };
            (piece2 as i8) * player2
        } else if let Ok(Square::Blank) = self.board.get_by_xy(x, y) {
            0
        } else {
            -99
        }
    }
}
