use super::entities::*;
use super::coords::*;
use super::bitboard::*;
use super::super::*;

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
        ml.write(MoveWithEval(MoveDescription::NormalMove(*origin, FastCoord(dest)), 0));
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
