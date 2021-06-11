use super::entities::*;
use super::move_list::*;
use super::board::*;
use super::coords::*;

// TODO Should be able to substitute bitboards for the same interface
// TODO Should also be able to generate bitboards with this class
pub struct BasicMoveTest<'a> {
    src_x: i8,
    src_y: i8,
    src_piece: Piece,
    src_player: Player,
    data: &'a Board,
    can_capture_king: bool,
    last_push_was_moveable: bool
}

impl <'a> BasicMoveTest<'a> {

    /// Pushes to `result` all the "basic" moves of a source piece with the special option to be able to capture a king
    pub fn fill_src(
        src_x: u8,
        src_y: u8,
        src_piece: Piece,
        src_player: Player,
        data: &Board,
        can_capture_king: bool,
        result: &mut MoveList
    ) {
        debug_assert!(src_x <= 7 && src_y <= 7);
        let mut t = BasicMoveTest {
            src_x: src_x as i8, src_y: src_y as i8, src_piece, src_player, data, can_capture_king, last_push_was_moveable: false
        };
        // TODO Array
        match src_piece {
            Piece::Pawn => t.push_pawn(result),
            Piece::Queen => t.push_queen(result),
            Piece::Knight => t.push_knight(result),
            Piece::King => t.push_king(result),
            Piece::Bishop => t.push_bishop(result),
            Piece::Rook => t.push_rook(result)
        }
    }

    /// Calls `fill_src` for all pieces owned by a player 
    pub fn fill_player(
        player_with_turn: Player,
        data: &Board,
        can_capture_king: bool,
        result: &mut MoveList
    ) {
        for Coord(x, y) in &data.get_player_state(player_with_turn).piece_locs {
            if let Square::Occupied(piece, player) = data.get_by_xy(*x, *y) {
                debug_assert!(player == player_with_turn);
                BasicMoveTest::fill_src(*x, *y, piece, player, data, can_capture_king, result);
            } else {
                panic!("Empty square in {:?} player's piece locs", player_with_turn);
            }
        }
    }

    pub fn has_king_capture_move(
        moves: &MoveList,
        start: usize,
        end_exclusive: usize,
        checked_player: Player
    ) -> bool {
        for j in start..end_exclusive {
            let modified_sqs = moves.get_v()[j];
            for sq_holder in modified_sqs.iter() {
                if let Some((_, BeforeAfterSquares(Square::Occupied(before_piece, before_player), Square::Occupied(_, after_player)))) = sq_holder {
                    if *before_piece == Piece::King && *before_player == checked_player && *after_player != checked_player {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    /// Filters out input moves which cannot be made because king will be captured
    pub fn filter_check_threats(
        real_board: &mut Board,
        checking_player: Player,

        candidates_and_buf: &mut MoveList,
        candidates_start: usize,
        candidates_end_exclusive: usize,

        result: &mut MoveList
    ) {
        let checked_player = checking_player.get_other_player();

        for i in candidates_start..candidates_end_exclusive {
            real_board.make_move(&candidates_and_buf.get_v()[i]);

            candidates_and_buf.write_index = candidates_end_exclusive;
            BasicMoveTest::fill_player(
                checking_player, 
                real_board,
                true,
                candidates_and_buf
            ); 
            let cand_write_end_exclusive = candidates_and_buf.write_index;

            if !BasicMoveTest::has_king_capture_move(candidates_and_buf, candidates_end_exclusive, cand_write_end_exclusive, checked_player) {
                let safe_move = candidates_and_buf.get_v()[i];
                result.write(safe_move);
            }

            real_board.undo_move(&candidates_and_buf.get_v()[i]);
        }
    }

    /// `can_capture` - eg. Pawn cannot capture forwards
    /// Returns whether the search along the same line should be terminated (if applicable)
    fn push(
        &mut self,
        test_dest_x: i8,
        test_dest_y: i8,
        can_capture: bool,
        replacement_piece: Option<Piece>,
        result: &mut MoveList
    ) -> bool {

        if test_dest_x < 0 || test_dest_x > 7 || test_dest_y < 0 || test_dest_y > 7 { 
            self.last_push_was_moveable = false;
            return true;
        }

        let (dest_x, dest_y) = (test_dest_x as u8, test_dest_y as u8);
        let existing_dest_sq = self.data.get_by_xy(dest_x, dest_y);
        let (moveable, terminate) = match existing_dest_sq {
            Square::Occupied(dest_piece, dest_square_player) => {(
                can_capture && dest_square_player != self.src_player && (self.can_capture_king || dest_piece != Piece::King), 
                true
            )},
            Square::Blank => {
                (true, false)
            }
        };

        //println!("{},{} moveable={} terminate={}", dest_x, dest_y, moveable, terminate);

        if moveable {
            result.write(self.make_move_snapshot(dest_x, dest_y, existing_dest_sq, replacement_piece));
        }

        self.last_push_was_moveable = moveable;
        return terminate;
    }

    /// Precondition: We have decided that this move is allowed, eg. finished evaluating ability to capture own pieces
    fn make_move_snapshot(
        &self,
        dest_x: u8,
        dest_y: u8,
        existing_dest_square: Square,
        replacement_piece: Option<Piece>
    ) -> MoveSnapshot {
        let mut m = MoveSnapshot::default();

        m.0[0] = Some((Coord(self.src_x as u8, self.src_y as u8), BeforeAfterSquares(
            Square::Occupied(self.src_piece, self.src_player),
            Square::Blank
        )));

        m.0[1] = Some((Coord(dest_x, dest_y), BeforeAfterSquares(
            existing_dest_square,
            Square::Occupied(replacement_piece.unwrap_or(self.src_piece), self.src_player)
        )));

        let mut first_prevented_oo = false; 
        let mut first_prevented_ooo = false;
        let player_state = self.data.get_player_state(self.src_player);
        if self.src_piece == Piece::Rook {
            first_prevented_oo = self.src_x == 7 && !player_state.moved_oo_piece;
            first_prevented_ooo = self.src_x == 0 && !player_state.moved_ooo_piece;
        } else if self.src_piece == Piece::King {
            first_prevented_oo = !player_state.moved_oo_piece;
            first_prevented_ooo = !player_state.moved_ooo_piece;
        }

        m.2 = if let Square::Occupied(_, _) = existing_dest_square {
            MoveDescription::Capture(first_prevented_oo, first_prevented_ooo, 1)
        } else {
            MoveDescription::Move(first_prevented_oo, first_prevented_ooo, 1)
        };

        return m;
    }

    fn push_promotions(&mut self, test_dest_x: i8, test_dest_y: i8, can_capture: bool, result: &mut MoveList) {
        self.push(test_dest_x, test_dest_y, can_capture, Some(Piece::Knight), result);
        if self.last_push_was_moveable {
            let existing_dest_sq = self.data.get_by_xy(test_dest_x as u8, test_dest_y as u8);
            result.write(self.make_move_snapshot(test_dest_x as u8, test_dest_y as u8, existing_dest_sq, Some(Piece::Queen))); 
            result.write(self.make_move_snapshot(test_dest_x as u8, test_dest_y as u8, existing_dest_sq, Some(Piece::Bishop))); 
            result.write(self.make_move_snapshot(test_dest_x as u8, test_dest_y as u8, existing_dest_sq, Some(Piece::Rook))); 
        }
    }

    fn push_pawn(&mut self, result: &mut MoveList) {
        let (y_delta, jump_row, pre_promote_row) = match self.src_player {
            Player::Black => (1, 1, 6),
            Player::White => (-1, 6, 1)
        };

        let (x, y) = (self.src_x as i8, self.src_y as i8);

        if y == pre_promote_row {
            self.push_promotions(x, y + y_delta, false, result);
        } else {
            if !self.push(x, y + y_delta, false, None, result) { // Same as rook ray. If 1-square hop is not blocked, consider 2-square hop.
                if y == jump_row {
                    self.push(x, y + y_delta * 2, false, None, result);
                }
            }
        }

        for x_delta in -1..=1 {
            if x_delta == 0 { continue; }

            let x_p_delta: i8 = x + x_delta;
            let y_p_delta: i8 = y + y_delta;

            if x_p_delta < 0 || x_p_delta > 7 { continue; }
            if y_p_delta < 0 || y_p_delta > 7 { continue; }

            if let Square::Occupied(_, angled_player) = self.data.get_by_xy(x_p_delta as u8, y_p_delta as u8) {
                if angled_player != self.src_player {
                    if y == pre_promote_row {
                        self.push_promotions(x + x_delta, y + y_delta, true, result);
                    } else {
                        self.push(x + x_delta, y + y_delta, true, None, result);
                    }
                }
            }
        }
    }

    fn push_rook(&mut self, result: &mut MoveList) {
        for _i in 1..=self.src_x {
            let i = self.src_x - _i;
            if self.push(i, self.src_y, true, None, result) { break; }
        }
        for i in self.src_x + 1..=7 {
            if self.push(i, self.src_y, true, None, result) { break; }
        }
        for _i in 1..=self.src_y {
            let i = self.src_y - _i;
            if self.push(self.src_x, i, true, None, result) { break; }
        }
        for i in self.src_y + 1..=7 {
            if self.push(self.src_x, i, true, None, result) { break; }
        }
    }

    fn push_bishop(&mut self, result: &mut MoveList) {

        for i in 1..=self.src_x {
            if self.push(self.src_x - i, self.src_y - i, true, None, result) { break; }
        }
        for i in 1..=self.src_x {
            if self.push(self.src_x - i, self.src_y + i, true, None, result) { break; }
        }
        for i in 1..=8 - (self.src_x + 1) {
            if self.push(self.src_x + i, self.src_y - i, true, None, result) { break; }
        }
        for i in 1..=8 - (self.src_x + 1) {
            if self.push(self.src_x + i, self.src_y + i, true, None, result) { break; }
        }
    }

    fn push_knight(&mut self, result: &mut MoveList) {

        self.push(self.src_x - 1, self.src_y + 2, true, None, result);
        self.push(self.src_x - 1, self.src_y - 2, true, None, result);

        self.push(self.src_x - 2, self.src_y + 1, true, None, result);
        self.push(self.src_x - 2, self.src_y - 1, true, None, result);

        self.push(self.src_x + 2, self.src_y + 1, true, None, result);
        self.push(self.src_x + 2, self.src_y - 1, true, None, result);

        self.push(self.src_x + 1, self.src_y + 2, true, None, result);
        self.push(self.src_x + 1, self.src_y - 2, true, None, result);
    }

    fn push_queen(&mut self, result: &mut MoveList) {
        self.push_bishop(result);
        self.push_rook(result);
    }

    fn push_king(&mut self, result: &mut MoveList) {
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 { continue; }
                self.push(self.src_x + i, self.src_y + j, true, None, result);
            }
        }
    }
}

