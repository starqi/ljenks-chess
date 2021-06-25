use super::move_test::*;
use super::entities::*;
use super::board::*;

pub struct CheckDetectionHandler {
    pub has_king_capture: bool
}

impl CheckDetectionHandler {
    pub fn new() -> CheckDetectionHandler {
        CheckDetectionHandler { has_king_capture: false }
    }
}

impl MoveTestHandler for CheckDetectionHandler {
    fn push(
        &mut self,
        moveable: bool,
        can_capture: bool,
        params: &MoveTestParams,
        dest_x: u8,
        dest_y: u8,
        existing_dest_square: &Square,
        replacement_piece: Option<Piece>
    ) -> bool {
        if moveable && can_capture {
            if let Square::Occupied(Piece::King, player) = existing_dest_square {
                if *player == params.src_player.get_other_player() {
                    self.has_king_capture = true;
                    return true;
                }
            }
        }
        return false;
    }
}

pub fn is_checking(real_board: &mut Board, checking_player: Player) -> bool {
    let mut handler = CheckDetectionHandler { has_king_capture: false };
    fill_player(checking_player, true, real_board, &mut handler); 
    handler.has_king_capture 
}
