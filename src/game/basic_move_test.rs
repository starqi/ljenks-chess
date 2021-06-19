use super::entities::*;
use super::move_list::*;
use super::board::*;
use super::coords::*;

// TODO Should be able to substitute bitboards for the same interface
// TODO Should also be able to generate bitboards with this class

pub trait MoveTestHandler {
    fn push(
        &mut self,
        moveable: bool,
        can_capture: bool,
        params: &BasicMoveTestParams,
        dest_x: u8,
        dest_y: u8,
        existing_dest_square: &Square,
        replacement_piece: Option<Piece>
    );
}

pub struct PushToMoveListHandler<'a> {
    pub move_list: &'a mut MoveList
}

impl <'a> MoveTestHandler for PushToMoveListHandler<'a> {

    fn push(
        &mut self,
        moveable: bool,
        can_capture: bool,
        params: &BasicMoveTestParams,
        dest_x: u8,
        dest_y: u8,
        existing_dest_square: &Square,
        replacement_piece: Option<Piece>
    ) {
        if !moveable { return; }

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
    }
}

//////////////////////////////////////////////////

struct Pusher<'a, T : MoveTestHandler> {
    /// eg. Pawn cannot capture forwards
    can_capture: bool,
    /// eg. Pawn can't move to empty at a diagonal
    can_move_to_empty: bool,
    /// For promotions
    replacement_piece: Option<Piece>,
    can_capture_king: bool,
    handler: &'a mut T
}

impl <'a, T : MoveTestHandler> Pusher<'a, T> {

    fn make_standard(can_capture_king: bool, handler: &mut T) -> Pusher<T> {
        Pusher {
            can_capture: true,
            can_move_to_empty: true,
            replacement_piece: None,
            can_capture_king,
            handler
        }
    }

    fn push(&mut self, x: i8, y: i8, params: &BasicMoveTestParams) -> (bool, bool) {

        if x < 0 || x > 7 || y < 0 || y > 7 { 
            return (false, true);
        }

        let existing_dest_sq = params.board.get_by_xy(x as u8, y as u8);
        let (moveable, terminate) = match existing_dest_sq {
            Square::Occupied(dest_piece, dest_square_player) => {(
                self.can_capture && *dest_square_player != params.src_player && (self.can_capture_king || *dest_piece != Piece::King), 
                true
            )},
            Square::Blank => {
                (self.can_move_to_empty, false)
            }
        };

        //crate::console_log!("{},{} moveable={} terminate={}", dest_x, dest_y, moveable, terminate);
        self.handler.push(moveable, self.can_capture, params, x as u8, y as u8, existing_dest_sq, self.replacement_piece);
        return (moveable, terminate);
    }
}

//////////////////////////////////////////////////

// Sets up a piece for some player at some square, replacing the one on the board,
// so we can get callbacks of the "basic" moves, eg. not involving checks, so that this functionality
// can support calculating checks.
pub struct BasicMoveTestParams<'a> {
    pub src_x: i8,
    pub src_y: i8,
    pub src_piece: Piece,
    pub src_player: Player,
    pub can_capture_king: bool,
    pub board: &'a Board
}

/// Pushes to `result` all the "basic" moves of a source piece with the special option to be able to capture a king
pub fn fill_src<T : MoveTestHandler>(params: &BasicMoveTestParams, handler: &mut T) {
    debug_assert!(params.src_x <= 7 && params.src_y <= 7 && params.src_x >= 0 && params.src_y >= 0);
    match params.src_piece {
        Piece::Pawn => push_pawn(params, handler),
        Piece::Queen => push_queen(params, handler),
        Piece::Knight => push_knight(params, handler),
        Piece::King => push_king(params, handler),
        Piece::Bishop => push_bishop(params, handler),
        Piece::Rook => push_rook(params, handler)
    }
}

/// Calls `fill_src` for all pieces owned by a player 
pub fn fill_player<T : MoveTestHandler>(
    player_with_turn: Player,
    can_capture_king: bool,
    board: &Board,
    handler: &mut T
) {
    for Coord(x, y) in &board.get_player_state(player_with_turn).piece_locs {
        if let Square::Occupied(piece, player) = board.get_by_xy(*x, *y) {
            debug_assert!(*player == player_with_turn);
            fill_src(&BasicMoveTestParams {
                src_x: *x as i8,
                src_y: *y as i8,
                src_piece: *piece,
                src_player: *player,
                board,
                can_capture_king
            }, handler);
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
        let modified_sqs = &moves.get_v()[j];
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

/// Filters out input moves which cannot be made because king will be captured.
/// Given move list will contain candidate moves, but beyond that, will be used as a temp buffer.
/// Precondition: `real_board` is ready to make the moves in the input move list.
pub fn filter_check_threats(
    real_board: &mut Board,

    candidates_and_buf: &mut MoveList,
    candidates_start: usize,
    candidates_end_exclusive: usize,

    result: &mut MoveList
) {
    let mut handler = PushToMoveListHandler { move_list: candidates_and_buf };
    let checked_player = real_board.get_player_with_turn();
    let checking_player = checked_player.get_other_player();

    for i in candidates_start..candidates_end_exclusive {
        let m_clone = handler.move_list.get_v()[i].clone();
        real_board.handle_move(&m_clone, true);

        handler.move_list.write_index = candidates_end_exclusive;
        fill_player(checking_player, true, real_board, &mut handler); 
        let second_end_exclusive = handler.move_list.write_index;

        let can_write = !has_king_capture_move(handler.move_list, candidates_end_exclusive, second_end_exclusive, checked_player);
        real_board.handle_move(&m_clone, false);

        if can_write { result.write(m_clone); }
    }
}

static PAWN_JUMP_ROWS: [(i8, i8, i8); 2] = [(-1, 6, 1), (1, 1, 6)];

fn push_pawn<T : MoveTestHandler>(params: &BasicMoveTestParams, handler: &mut T) {

    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    let (y_delta, jump_row, pre_promote_row) = PAWN_JUMP_ROWS[params.src_player as usize];
    let (x, y) = (params.src_x as i8, params.src_y as i8);

    if y == pre_promote_row {
        pusher.can_capture = false;
        push_promotions(&mut pusher, x, y + y_delta, params);

        pusher.can_capture = true;
        pusher.can_move_to_empty = false;
        for x_delta in -1..=1 {
            if x_delta == 0 { continue; }
            push_promotions(&mut pusher, x + x_delta, y + y_delta, params);
        }
    } else {
        pusher.can_capture = false;
        if !pusher.push(x, y + y_delta, params).1 { // Same as rook ray. If 1-square hop is not blocked, consider 2-square hop.
            if y == jump_row {
                pusher.push(x, y + y_delta * 2, params);
            }
        }

        pusher.can_capture = true;
        pusher.can_move_to_empty = false;
        for x_delta in -1..=1 {
            if x_delta == 0 { continue; }
            pusher.push(x + x_delta, y + y_delta, params);
        }
    }
}

fn push_rook<T : MoveTestHandler>(params: &BasicMoveTestParams, handler: &mut T) {
    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    for _i in 1..=params.src_x {
        let i = params.src_x - _i;
        if pusher.push(i, params.src_y, params).1 { break; }
    }
    for i in params.src_x + 1..=7 {
        if pusher.push(i, params.src_y, params).1 { break; }
    }
    for _i in 1..=params.src_y {
        let i = params.src_y - _i;
        if pusher.push(params.src_x, i, params).1 { break; }
    }
    for i in params.src_y + 1..=7 {
        if pusher.push(params.src_x, i, params).1 { break; }
    }
}

fn push_bishop<T : MoveTestHandler>(params: &BasicMoveTestParams, handler: &mut T) {
    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    for i in 1..=params.src_x {
        if pusher.push(params.src_x - i, params.src_y - i, params).1 { break; }
    }
    for i in 1..=params.src_x {
        if pusher.push(params.src_x - i, params.src_y + i, params).1 { break; }
    }
    for i in 1..=8 - (params.src_x + 1) {
        if pusher.push(params.src_x + i, params.src_y - i, params).1 { break; }
    }
    for i in 1..=8 - (params.src_x + 1) {
        if pusher.push(params.src_x + i, params.src_y + i, params).1 { break; }
    }
}

fn push_knight<T : MoveTestHandler>(params: &BasicMoveTestParams, handler: &mut T) {
    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    pusher.push(params.src_x - 1, params.src_y + 2, params);
    pusher.push(params.src_x - 1, params.src_y - 2, params);

    pusher.push(params.src_x - 2, params.src_y + 1, params);
    pusher.push(params.src_x - 2, params.src_y - 1, params);

    pusher.push(params.src_x + 2, params.src_y + 1, params);
    pusher.push(params.src_x + 2, params.src_y - 1, params);

    pusher.push(params.src_x + 1, params.src_y + 2, params);
    pusher.push(params.src_x + 1, params.src_y - 2, params);
}

fn push_queen<T : MoveTestHandler>(params: &BasicMoveTestParams, handler: &mut T) {
    push_bishop(params, handler);
    push_rook(params, handler);
}

fn push_king<T : MoveTestHandler>(params: &BasicMoveTestParams, handler: &mut T) {
    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 { continue; }
            pusher.push(params.src_x + i, params.src_y + j, params);
        }
    }
}

/// Will overwrite `handler` config
fn push_promotions<T : MoveTestHandler>(
    pusher: &mut Pusher<T>, x: i8, y: i8, params: &BasicMoveTestParams
) -> (bool, bool) {

    pusher.replacement_piece = Some(Piece::Knight);

    let (moveable, terminate) = pusher.push(x, y, params);

    if moveable {
        pusher.replacement_piece = Some(Piece::Rook);
        pusher.push(x, y, params);

        pusher.replacement_piece = Some(Piece::Queen);
        pusher.push(x, y, params); 

        pusher.replacement_piece = Some(Piece::Bishop);
        pusher.push(x, y, params);
    }

    (moveable, terminate)
}
