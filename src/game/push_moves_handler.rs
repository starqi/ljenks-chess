use super::move_test::*;
use super::entities::*;
use super::coords::*;
use super::move_list::*;

pub struct PushToMoveListHandler<'a> {
    pub move_list: &'a mut MoveList
}

impl <'a> MoveTestHandler for PushToMoveListHandler<'a> {

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
        if !moveable { return false; }

        let mut m = MoveSnapshot::default();

        m.0[0] = Some((Coord(params.src_x as u8, params.src_y as u8), BeforeAfterSquares(
            Square::Occupied(params.src_piece, params.src_player),
            Square::Blank
        )));

        m.0[1] = Some((Coord(dest_x, dest_y), BeforeAfterSquares(
            *existing_dest_square,
            Square::Occupied(replacement_piece.unwrap_or(params.src_piece), params.src_player)
        )));

        let mut first_prevented_oo = false; 
        let mut first_prevented_ooo = false;

        let player_state = params.board.get_player_state(params.src_player);
        if params.src_piece == Piece::Rook {
            first_prevented_oo = params.src_x == 7 && !player_state.moved_oo_piece;
            first_prevented_ooo = params.src_x == 0 && !player_state.moved_ooo_piece;
        } else if params.src_piece == Piece::King {
            first_prevented_oo = !player_state.moved_oo_piece;
            first_prevented_ooo = !player_state.moved_ooo_piece;
        }

        // Since we are dealing with "basic" moves, there are only captures and moves
        m.2 = if let Square::Occupied(_, _) = existing_dest_square {
            MoveDescription::Capture(first_prevented_oo, first_prevented_ooo, 1)
        } else {
            MoveDescription::Move(first_prevented_oo, first_prevented_ooo, 1)
        };

        self.move_list.write(m);
        return false;
    }
}

//////////////////////////////////////////////////
