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

        let move_desc = MoveDescription::NormalMove(
            FastCoord::from_coords(params.src_x as u8, params.src_y as u8),
            FastCoord::from_coords(dest_x, dest_y)
        );

        let m = MoveWithEval(move_desc, 0.0);

        self.move_list.write(m);
        return false;
    }
}
