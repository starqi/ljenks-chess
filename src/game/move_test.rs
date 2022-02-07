use super::entities::*;
use super::coords::*;
use super::bitboard::*;
use super::super::*;

pub trait MoveTestHandler {
    /// Returns whether to force terminate the whole piece, do not rely on not being called again if true
    fn push(
        &mut self,
        moveable: bool,
        can_capture: bool,
        params: &MoveTestParams,
        dest_x: u8,
        dest_y: u8,
        existing_dest_square: &Square,
        replacement_piece: Option<Piece>
    ) -> bool;
}

struct Pusher<'a, T : MoveTestHandler> {
    /// eg. Pawn cannot capture forwards
    can_capture: bool,
    /// eg. Pawn can't move to empty at a diagonal
    can_move_to_empty: bool,
    /// For promotions
    replacement_piece: Option<Piece>,
    can_capture_king: bool,
    terminate_all_flag: bool,
    handler: &'a mut T
}

impl <'a, T : MoveTestHandler> Pusher<'a, T> {

    fn make_standard(can_capture_king: bool, handler: &mut T) -> Pusher<T> {
        Pusher {
            can_capture: true,
            can_move_to_empty: true,
            replacement_piece: None,
            can_capture_king,
            terminate_all_flag: false,
            handler
        }
    }

    fn push(&mut self, x: i8, y: i8, params: &MoveTestParams) -> (bool, bool) {

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
        self.terminate_all_flag |= self.handler.push(moveable, self.can_capture, params, x as u8, y as u8, existing_dest_sq, self.replacement_piece);
        return (moveable, terminate || self.terminate_all_flag);
    }
}

//////////////////////////////////////////////////

// Sets up a piece for some player at some square, replacing the one on the board,
// so we can get callbacks of the "basic" moves, eg. not involving checks, so that this functionality
// can support calculating checks.
pub struct MoveTestParams<'a> {
    pub src_x: i8,
    pub src_y: i8,
    pub src_piece: Piece,
    pub src_player: Player,
    pub can_capture_king: bool,
    pub board: &'a Board
}

/// Pushes to `result` all the "basic" moves of a source piece with the special option to be able to capture a king
pub fn fill_src<T : MoveTestHandler>(params: &MoveTestParams, handler: &mut T) {
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
    let mut piece_locs = board.get_player_state(player_with_turn).piece_locs;
    piece_locs.consume_loop_indices(|index| {
        if let Square::Occupied(piece, player) = board.get_by_index(index) {
            debug_assert!(*player == player_with_turn, "Player owns a wrong colored piece");
            let coord = FastCoord(index).to_coord();
            fill_src(&MoveTestParams {
                src_x: coord.0 as i8,
                src_y: coord.1 as i8,
                src_piece: *piece,
                src_player: *player,
                board,
                can_capture_king
            }, handler);
        } else {
            panic!("Empty square in {:?} player's piece locs", player_with_turn);
        }
    });
}

static PAWN_JUMP_ROWS: [(i8, i8, i8); 2] = [(-1, 6, 1), (1, 1, 6)];

fn push_pawn<T : MoveTestHandler>(params: &MoveTestParams, handler: &mut T) {

    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    let (y_delta, jump_row, pre_promote_row) = PAWN_JUMP_ROWS[params.src_player as usize];
    let (x, y) = (params.src_x as i8, params.src_y as i8);

    if y == pre_promote_row {
        pusher.can_capture = false;
        push_promotions(&mut pusher, x, y + y_delta, params);
        if pusher.terminate_all_flag { return; }

        pusher.can_capture = true;
        pusher.can_move_to_empty = false;
        for x_delta in -1..=1 {
            if x_delta == 0 { continue; }
            push_promotions(&mut pusher, x + x_delta, y + y_delta, params);
            if pusher.terminate_all_flag { return; }
        }
    } else {
        pusher.can_capture = false;
        if !pusher.push(x, y + y_delta, params).1 { // Same as rook ray. If 1-square hop is not blocked, consider 2-square hop.
            if y == jump_row {
                pusher.push(x, y + y_delta * 2, params);
            }
        }
        if pusher.terminate_all_flag { return; }

        pusher.can_capture = true;
        pusher.can_move_to_empty = false;
        for x_delta in -1..=1 {
            if x_delta == 0 { continue; }
            pusher.push(x + x_delta, y + y_delta, params);
            if pusher.terminate_all_flag { return; }
        }
    }
}

fn push_rook<T : MoveTestHandler>(params: &MoveTestParams, handler: &mut T) {
    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    for _i in 1..=params.src_x {
        let i = params.src_x - _i;
        if pusher.push(i, params.src_y, params).1 { break; }
    }
    if pusher.terminate_all_flag { return; }

    for i in params.src_x + 1..=7 {
        if pusher.push(i, params.src_y, params).1 { break; }
    }
    if pusher.terminate_all_flag { return; }

    for _i in 1..=params.src_y {
        let i = params.src_y - _i;
        if pusher.push(params.src_x, i, params).1 { break; }
    }
    if pusher.terminate_all_flag { return; }

    for i in params.src_y + 1..=7 {
        if pusher.push(params.src_x, i, params).1 { break; }
    }
    if pusher.terminate_all_flag { return; }
}

fn push_bishop<T : MoveTestHandler>(params: &MoveTestParams, handler: &mut T) {
    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    for i in 1..=params.src_x {
        if pusher.push(params.src_x - i, params.src_y - i, params).1 { break; }
    }
    if pusher.terminate_all_flag { return; }

    for i in 1..=params.src_x {
        if pusher.push(params.src_x - i, params.src_y + i, params).1 { break; }
    }
    if pusher.terminate_all_flag { return; }

    for i in 1..=8 - (params.src_x + 1) {
        if pusher.push(params.src_x + i, params.src_y - i, params).1 { break; }
    }
    if pusher.terminate_all_flag { return; }

    for i in 1..=8 - (params.src_x + 1) {
        if pusher.push(params.src_x + i, params.src_y + i, params).1 { break; }
    }
    if pusher.terminate_all_flag { return; }
}

fn push_knight<T : MoveTestHandler>(params: &MoveTestParams, handler: &mut T) {
    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    pusher.push(params.src_x - 1, params.src_y + 2, params);
    if pusher.terminate_all_flag { return; }
    pusher.push(params.src_x - 1, params.src_y - 2, params);
    if pusher.terminate_all_flag { return; }

    pusher.push(params.src_x - 2, params.src_y + 1, params);
    if pusher.terminate_all_flag { return; }
    pusher.push(params.src_x - 2, params.src_y - 1, params);
    if pusher.terminate_all_flag { return; }

    pusher.push(params.src_x + 2, params.src_y + 1, params);
    if pusher.terminate_all_flag { return; }
    pusher.push(params.src_x + 2, params.src_y - 1, params);
    if pusher.terminate_all_flag { return; }

    pusher.push(params.src_x + 1, params.src_y + 2, params);
    if pusher.terminate_all_flag { return; }
    pusher.push(params.src_x + 1, params.src_y - 2, params);
    if pusher.terminate_all_flag { return; }
}

fn push_queen<T : MoveTestHandler>(params: &MoveTestParams, handler: &mut T) {
    push_bishop(params, handler);
    push_rook(params, handler);
}

fn push_king<T : MoveTestHandler>(params: &MoveTestParams, handler: &mut T) {
    let mut pusher = Pusher::make_standard(params.can_capture_king, handler);

    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 { continue; }
            pusher.push(params.src_x + i, params.src_y + j, params);
            if pusher.terminate_all_flag { return; }
        }
    }
}

/// Will overwrite `handler` config
fn push_promotions<T : MoveTestHandler>(
    pusher: &mut Pusher<T>, x: i8, y: i8, params: &MoveTestParams
) -> (bool, bool) {

    pusher.replacement_piece = Some(Piece::Knight);

    let (moveable, terminate) = pusher.push(x, y, params);
    if pusher.terminate_all_flag {
        return (moveable, terminate);
    }

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

fn set_blockable_ray(
    b: &mut Bitboard,
    origin: FastCoord,
    direction: RayDirection,
    ensure_blocker_index: usize,
    get_first_blocker_index: impl FnOnce(&Bitboard) -> u8,
    blockers: &Bitboard
) {
    let direction_num = direction as usize;
    let ray = BITBOARD_PRESETS.rays[direction_num][origin.0 as usize];
    // There will always be at least 1 blocker, even for an empty board
    let blocked_at = Bitboard((ray.0 & blockers.0) | BITBOARD_PRESETS.ensure_blocker[ensure_blocker_index].0);
    let first_blocked_at_index = get_first_blocker_index(&blocked_at);
    let past_blocked_ray = BITBOARD_PRESETS.rays[direction_num][first_blocked_at_index as usize].0;
    let moves = Bitboard(past_blocked_ray ^ ray.0);
    b.0 |= moves.0;
}

fn consume_to_move_list(b: &mut Bitboard, ml: &mut MoveList, origin: &FastCoord) {
    b.consume_loop_indices(|dest| {
        ml.write(MoveWithEval(MoveDescription::NormalMove(*origin, FastCoord(dest)), 0.0));
    });
}

#[inline]
fn hits_king(b: &Bitboard, king_location: &Bitboard) -> bool {
    (b.0 & king_location.0) != 0
}

#[inline]
fn unset_own_pieces(b: &mut Bitboard, curr_player_piece_locs: &Bitboard) {
    b.0 &= !curr_player_piece_locs.0;
}

#[inline]
fn _write_rook_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) -> Bitboard {
    let blockers = Bitboard(curr_player_piece_locs.0 | opponent_piece_locs.0);
    
    let mut b = Bitboard(0);
    set_blockable_ray(&mut b, origin, RayDirection::Left, 0, Bitboard::_lsb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::Top, 0, Bitboard::_lsb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::Right, 1, Bitboard::_msb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::Bottom, 1, Bitboard::_msb_to_index, &blockers);
    unset_own_pieces(&mut b, curr_player_piece_locs);
    b
}

pub fn write_rook_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_rook_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    consume_to_move_list(&mut b, ml, &origin);
}

pub fn rook_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_rook_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

#[inline]
fn _write_bishop_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) -> Bitboard {
    let blockers = Bitboard(curr_player_piece_locs.0 | opponent_piece_locs.0);

    let mut b = Bitboard(0);
    set_blockable_ray(&mut b, origin, RayDirection::LeftTop, 0, Bitboard::_lsb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::RightTop, 0, Bitboard::_lsb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::RightBottom, 1, Bitboard::_msb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::LeftBottom, 1, Bitboard::_msb_to_index, &blockers);
    unset_own_pieces(&mut b, curr_player_piece_locs);
    b
}

pub fn write_bishop_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_bishop_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    consume_to_move_list(&mut b, ml, &origin);
}

pub fn bishop_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_bishop_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

#[inline]
fn _write_queen_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) -> Bitboard {
    let blockers = Bitboard(curr_player_piece_locs.0 | opponent_piece_locs.0);

    let mut b = Bitboard(0);
    set_blockable_ray(&mut b, origin, RayDirection::LeftTop, 0, Bitboard::_lsb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::RightTop, 0, Bitboard::_lsb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::RightBottom, 1, Bitboard::_msb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::LeftBottom, 1, Bitboard::_msb_to_index, &blockers);
    
    set_blockable_ray(&mut b, origin, RayDirection::Left, 0, Bitboard::_lsb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::Top, 0, Bitboard::_lsb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::Right, 1, Bitboard::_msb_to_index, &blockers);
    set_blockable_ray(&mut b, origin, RayDirection::Bottom, 1, Bitboard::_msb_to_index, &blockers);
    unset_own_pieces(&mut b, curr_player_piece_locs);
    b
}

pub fn write_queen_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_queen_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    consume_to_move_list(&mut b, ml, &origin);
}

pub fn queen_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_queen_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

#[inline]
fn _write_knight_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard) -> Bitboard {
    let mut jumps = Bitboard(BITBOARD_PRESETS.knight_jumps[origin.value() as usize].0);
    unset_own_pieces(&mut jumps, curr_player_piece_locs);
    jumps
}

pub fn write_knight_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard) {
    let mut b = _write_knight_moves(origin, curr_player_piece_locs);
    consume_to_move_list(&mut b, ml, &origin);
}

pub fn knight_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_knight_moves(origin, curr_player_piece_locs);
    hits_king(&b, opponent_king_location)
}

#[inline]
fn _write_king_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard) -> Bitboard {
    let mut m = Bitboard(BITBOARD_PRESETS.king_moves[origin.value() as usize].0);
    unset_own_pieces(&mut m, curr_player_piece_locs);
    m
}

pub fn write_king_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard) {
    let mut b = _write_king_moves(origin, curr_player_piece_locs);
    consume_to_move_list(&mut b, ml, &origin);
}

// This is a case, this is what prevents kings from getting close
pub fn king_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_king_moves(origin, curr_player_piece_locs);
    hits_king(&b, opponent_king_location)
}

#[inline]
fn _write_pawn_moves(
    origin: FastCoord,
    slide_push_blockers: impl FnOnce(u64) -> u64,
    curr_player: Player,
    curr_player_piece_locs: &Bitboard,
    opponent_piece_locs: &Bitboard
) -> Bitboard {
    let curr_player_num = curr_player as usize;
    let index = origin.value() as usize;

    // Blockers which block pushes, ie. everyone's pieces, excluding the current pawn
    let push_blockers_without_pawn = Bitboard((curr_player_piece_locs.0 | opponent_piece_locs.0) & !(1u64 << (63 - origin.0))).0;
    // Do a "motion blur" of the blockers towards the opponent direction, and convert to a mask
    let push_blockers_without_pawn2_mask = !(slide_push_blockers(push_blockers_without_pawn) | push_blockers_without_pawn);
    // This should handle blockers for pushes
    let push_locs = BITBOARD_PRESETS.pawn_pushes[curr_player_num][index].0 & push_blockers_without_pawn2_mask;
    let capture_locs = BITBOARD_PRESETS.pawn_captures[curr_player_num][index].0 & opponent_piece_locs.0;
    Bitboard(push_locs | capture_locs)
}

#[inline]
fn _write_white_pawn_moves(
    origin: FastCoord,
    piece_locs: &Bitboard,
    opponent_piece_locs: &Bitboard
) -> Bitboard {
    _write_pawn_moves(origin, |blockers| blockers << 8, Player::White, piece_locs, opponent_piece_locs)
}

pub fn write_white_pawn_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_white_pawn_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    consume_to_move_list(&mut b, ml, &origin);
}

pub fn white_pawn_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_white_pawn_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

#[inline]
fn _write_black_pawn_moves(
    origin: FastCoord,
    piece_locs: &Bitboard,
    opponent_piece_locs: &Bitboard
) -> Bitboard {
    _write_pawn_moves(origin, |blockers| blockers >> 8, Player::Black, piece_locs, opponent_piece_locs)
}

pub fn write_black_pawn_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_black_pawn_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    consume_to_move_list(&mut b, ml, &origin);
}

pub fn black_pawn_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_black_pawn_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

#[cfg(test)]
mod test {

    use super::*;

    #[ignore]
    #[test]
    fn moves_eyeball_test() {
        let origin = FastCoord::from_xy(3, 3);

        let mut blockers = Bitboard(0);
        blockers.set(2, 2);
        blockers.set(6, 6);
        blockers.set(5, 5);
        blockers.set(1, 1);

        let mut b = Bitboard(0);
        set_blockable_ray(&mut b, origin, RayDirection::LeftTop, 0, Bitboard::_lsb_to_index, &blockers);
        set_blockable_ray(&mut b, origin, RayDirection::RightTop, 0, Bitboard::_lsb_to_index, &blockers);
        set_blockable_ray(&mut b, origin, RayDirection::RightBottom, 1, Bitboard::_msb_to_index, &blockers);
        set_blockable_ray(&mut b, origin, RayDirection::LeftBottom, 1, Bitboard::_msb_to_index, &blockers);
    
        println!("{}", b);
    }

    #[ignore]
    #[test]
    fn moves_eyeball_test2() {
        let origin = FastCoord::from_xy(0, 7);

        let mut blockers = Bitboard(0);
        blockers.set(1, 7);

        let mut b = Bitboard(0);
        set_blockable_ray(&mut b, origin, RayDirection::Left, 0, Bitboard::_lsb_to_index, &blockers);
        set_blockable_ray(&mut b, origin, RayDirection::Top, 0, Bitboard::_lsb_to_index, &blockers);
        set_blockable_ray(&mut b, origin, RayDirection::Right, 1, Bitboard::_msb_to_index, &blockers);
        set_blockable_ray(&mut b, origin, RayDirection::Bottom, 1, Bitboard::_msb_to_index, &blockers);
        println!("{}", b);
    }
}
