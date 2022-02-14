use super::entities::*;
use super::coords::*;
use super::bitboard::*;
use super::super::*;

pub struct AttackFromBoards {
    pub data: [Bitboard; 64]
}

impl AttackFromBoards {
    pub fn new() -> AttackFromBoards {
        AttackFromBoards { data: [Bitboard(0); 64] }
    }

    pub fn reset(&mut self) {
        self.data.fill(Bitboard(0));
    }
}

pub struct CheckCaptureParams<'a> {
    pub curr_player_piece_locs: &'a Bitboard,
    pub opponent_piece_locs: &'a Bitboard,
    pub king_potential_rook_atks: Bitboard,
    pub king_potential_bishop_atks: Bitboard,
    pub king_potential_knight_atks: Bitboard,
    pub king_potential_pawn_atks: Bitboard
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

fn consume_to_move_list(b: &mut Bitboard, origin: FastCoord, result: &mut MoveList) {
    b.consume_loop_indices(|dest| {
        result.write(MoveWithEval(MoveDescription::NormalMove(origin, FastCoord(dest)), 0));
    });
}

fn update_attack_from_boards(origin: FastCoord, b: &mut Bitboard, result: &mut AttackFromBoards) {
    b.consume_loop_indices(|dest| {
        result.data[dest as usize].set_index(origin.0);
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

//////////////////////////////////////////////////
// Rook 

#[inline]
pub fn _write_rook_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) -> Bitboard {
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
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_rook_ccs(ml: &mut MoveList, origin: FastCoord, params: &CheckCaptureParams) {
    let mut b = _write_rook_moves(origin, params.curr_player_piece_locs, params.opponent_piece_locs);
    b.0 &= params.opponent_piece_locs.0 | params.king_potential_rook_atks.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn update_rook_af(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, result: &mut AttackFromBoards) {
    let mut b = _write_rook_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    update_attack_from_boards(origin, &mut b, result);
}

pub fn rook_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_rook_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

//////////////////////////////////////////////////
// Bishop 

#[inline]
pub fn _write_bishop_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) -> Bitboard {
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
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_bishop_ccs(ml: &mut MoveList, origin: FastCoord, params: &CheckCaptureParams) {
    let mut b = _write_bishop_moves(origin, &params.curr_player_piece_locs, &params.opponent_piece_locs);
    b.0 &= params.opponent_piece_locs.0 | params.king_potential_bishop_atks.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn update_bishop_af(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, result: &mut AttackFromBoards) {
    let mut b = _write_bishop_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    update_attack_from_boards(origin, &mut b, result);
}

pub fn bishop_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_bishop_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

//////////////////////////////////////////////////
// Queen 

#[inline]
pub fn _write_queen_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) -> Bitboard {
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
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_queen_captures(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_queen_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    b.0 &= opponent_piece_locs.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_queen_ccs(ml: &mut MoveList, origin: FastCoord, params: &CheckCaptureParams) {
    let mut b = _write_queen_moves(origin, &params.curr_player_piece_locs, &params.opponent_piece_locs);
    b.0 &= params.opponent_piece_locs.0 | params.king_potential_bishop_atks.0 | params.king_potential_rook_atks.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn update_queen_af(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, result: &mut AttackFromBoards) {
    let mut b = _write_queen_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    update_attack_from_boards(origin, &mut b, result);
}

pub fn queen_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_queen_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

//////////////////////////////////////////////////
// Knight 

#[inline]
pub fn _write_knight_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard) -> Bitboard {
    let mut jumps = Bitboard(BITBOARD_PRESETS.knight_jumps[origin.value() as usize].0);
    unset_own_pieces(&mut jumps, curr_player_piece_locs);
    jumps
}

pub fn write_knight_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard) {
    let mut b = _write_knight_moves(origin, curr_player_piece_locs);
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_knight_captures(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_knight_moves(origin, curr_player_piece_locs);
    b.0 &= opponent_piece_locs.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_knight_ccs(ml: &mut MoveList, origin: FastCoord, params: &CheckCaptureParams) {
    let mut b = _write_knight_moves(origin, &params.curr_player_piece_locs);
    b.0 &= params.opponent_piece_locs.0 | params.king_potential_knight_atks.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn update_knight_af(origin: FastCoord, curr_player_piece_locs: &Bitboard, result: &mut AttackFromBoards) {
    let mut b = _write_knight_moves(origin, curr_player_piece_locs);
    update_attack_from_boards(origin, &mut b, result);
}

pub fn knight_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_knight_moves(origin, curr_player_piece_locs);
    hits_king(&b, opponent_king_location)
}

//////////////////////////////////////////////////
// King 

#[inline]
pub fn _write_king_moves(origin: FastCoord, curr_player_piece_locs: &Bitboard) -> Bitboard {
    let mut m = Bitboard(BITBOARD_PRESETS.king_moves[origin.value() as usize].0);
    unset_own_pieces(&mut m, curr_player_piece_locs);
    m
}

pub fn write_king_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard) {
    let mut b = _write_king_moves(origin, curr_player_piece_locs);
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_king_captures(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_king_moves(origin, curr_player_piece_locs);
    b.0 &= opponent_piece_locs.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn update_king_af(origin: FastCoord, curr_player_piece_locs: &Bitboard, result: &mut AttackFromBoards) {
    let mut b = _write_king_moves(origin, curr_player_piece_locs);
    update_attack_from_boards(origin, &mut b, result);
}

// This is a case, this is what prevents kings from getting close
pub fn king_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_king_moves(origin, curr_player_piece_locs);
    hits_king(&b, opponent_king_location)
}

//////////////////////////////////////////////////
// Pawn 

#[inline]
fn _write_pawn_captures(
    origin: FastCoord,
    curr_player: Player,
    opponent_piece_locs: &Bitboard
) -> Bitboard {
    let curr_player_num = curr_player as usize;
    let index = origin.value() as usize;
    Bitboard(BITBOARD_PRESETS.pawn_captures[curr_player_num][index].0 & opponent_piece_locs.0)
}

// Not public, don't expose `slide_push_blockers` internal,
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
    let capture_locs = _write_pawn_captures(origin, curr_player, opponent_piece_locs);
    Bitboard(push_locs | capture_locs.0)
}

#[inline]
pub fn _write_white_pawn_moves(
    origin: FastCoord,
    piece_locs: &Bitboard,
    opponent_piece_locs: &Bitboard
) -> Bitboard {
    _write_pawn_moves(origin, |blockers| blockers << 8, Player::White, piece_locs, opponent_piece_locs)
}

#[inline]
pub fn _write_white_pawn_captures(
    origin: FastCoord,
    opponent_piece_locs: &Bitboard
) -> Bitboard {
    _write_pawn_captures(origin, Player::White, opponent_piece_locs)
}

pub fn write_white_pawn_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_white_pawn_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_white_pawn_ccs(ml: &mut MoveList, origin: FastCoord, params: &CheckCaptureParams) {
    let mut b = _write_white_pawn_captures(origin, &params.opponent_piece_locs);
    b.0 |= _write_white_pawn_moves(origin, &params.curr_player_piece_locs, &params.opponent_piece_locs).0 & params.king_potential_pawn_atks.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn update_white_pawn_af(origin: FastCoord, opponent_piece_locs: &Bitboard, result: &mut AttackFromBoards) {
    let mut b = _write_white_pawn_captures(origin, opponent_piece_locs);
    update_attack_from_boards(origin, &mut b, result);
}

pub fn white_pawn_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_white_pawn_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

#[inline]
pub fn _write_black_pawn_moves(
    origin: FastCoord,
    piece_locs: &Bitboard,
    opponent_piece_locs: &Bitboard
) -> Bitboard {
    _write_pawn_moves(origin, |blockers| blockers >> 8, Player::Black, piece_locs, opponent_piece_locs)
}

#[inline]
pub fn _write_black_pawn_captures(
    origin: FastCoord,
    opponent_piece_locs: &Bitboard
) -> Bitboard {
    _write_pawn_captures(origin, Player::Black, opponent_piece_locs)
}

pub fn write_black_pawn_moves(ml: &mut MoveList, origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard) {
    let mut b = _write_black_pawn_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    consume_to_move_list(&mut b, origin, ml);
}

pub fn write_black_pawn_ccs(ml: &mut MoveList, origin: FastCoord, params: &CheckCaptureParams) {
    let mut b = _write_black_pawn_captures(origin, &params.opponent_piece_locs);
    b.0 |= _write_black_pawn_moves(origin, &params.curr_player_piece_locs, &params.opponent_piece_locs).0 & params.king_potential_pawn_atks.0;
    consume_to_move_list(&mut b, origin, ml);
}

pub fn update_black_pawn_af(origin: FastCoord, opponent_piece_locs: &Bitboard, result: &mut AttackFromBoards) {
    let mut b = _write_black_pawn_captures(origin, opponent_piece_locs);
    update_attack_from_boards(origin, &mut b, result);
}

pub fn black_pawn_hits_king(origin: FastCoord, curr_player_piece_locs: &Bitboard, opponent_piece_locs: &Bitboard, opponent_king_location: &Bitboard) -> bool {
    let b = _write_black_pawn_moves(origin, curr_player_piece_locs, opponent_piece_locs);
    hits_king(&b, opponent_king_location)
}

//////////////////////////////////////////////////
// Tests 

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
