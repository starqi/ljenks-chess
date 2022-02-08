use super::bitboard::*;

#[repr(u8)]
pub enum RayDirection {
    Left = 0, LeftTop, Top, RightTop, Right, RightBottom, Bottom, LeftBottom
}

pub struct BitboardPresets {
    /// Index = `RayDirection` enum order
    pub rays: [[Bitboard; 64]; 8],
    pub knight_jumps: [Bitboard; 64],
    /// Array index = `Player` enum order
    pub pawn_pushes: [[Bitboard; 64]; 2],
    /// Array index = `Player` enum order
    pub pawn_captures: [[Bitboard; 64]; 2],
    pub king_moves: [Bitboard; 64],
    /// Array index = LSB, MSB
    pub ensure_blocker: [Bitboard; 2],
    pub debruijn_indices: [u8; 64],
    pub debruijn_sequence: u64
}

impl BitboardPresets {
    pub fn new() -> BitboardPresets {
        BitboardPresets {
            rays: [
                make_ray_lookup(-1, 0), make_ray_lookup(-1, -1), make_ray_lookup(0, -1), make_ray_lookup(1, -1),
                make_ray_lookup(1, 0), make_ray_lookup(1, 1), make_ray_lookup(0, 1), make_ray_lookup(-1, 1)
            ],
            knight_jumps: make_knight_jump_lookup(),
            pawn_pushes: [make_pawn_lookup(-1, 6), make_pawn_lookup(1, 1)],
            pawn_captures: [make_pawn_capture_lookup(-1), make_pawn_capture_lookup(1)],
            king_moves: make_king_lookup(),
            ensure_blocker: [Bitboard(1u64 << 63), Bitboard(1)],

            // Use existing sequence but convert to index 0 = a8, https://www.chessprogramming.org/BitScan#De_Bruijn_Multiplication
            debruijn_indices: [63, 62, 15, 61, 6, 14, 35, 60, 2, 5, 13, 21, 25, 34, 46, 59, 1, 8, 4, 27, 10, 12, 20, 41, 18, 24, 30, 33, 39, 45, 51, 58, 0, 16, 7, 36, 3, 22, 26, 47, 9, 28, 11, 42, 19, 31, 40, 52, 17, 37, 23, 48, 29, 43, 32, 53, 38, 49, 44, 54, 50, 55, 56, 57],
            debruijn_sequence: 0x03f79d71b4cb0a89
        }
    }
}

fn make_king_lookup() -> [Bitboard; 64] {
    let mut result = [Bitboard(0); 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut b = Bitboard(0);

            for i in -1..=1 {
                for j in -1..=1 {
                    if i == 0 && j == 0 { continue; }
                    b.slow_safe_set(x + i, y + j);
                }
            }
            result[(y * 8 + x) as usize].0 = b.0;
        }
    }
    result
}

fn make_pawn_capture_lookup(dy: i8) -> [Bitboard; 64] {
    let mut result = [Bitboard(0); 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut b = Bitboard(0);
            b.slow_safe_set(x - 1, y + dy);
            b.slow_safe_set(x + 1, y + dy);
            result[(y * 8 + x) as usize].0 = b.0;
        }
    }
    result
}

fn make_pawn_lookup(dy: i8, jump_y: i8) -> [Bitboard; 64] {
    let mut result = [Bitboard(0); 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut b = Bitboard(0);
            b.slow_safe_set(x, y + dy);
            if y == jump_y {
                b.slow_safe_set(x, y + dy + dy);
            }
            result[(y * 8 + x) as usize].0 = b.0;
        }
    }
    result
}

fn make_knight_jump_lookup() -> [Bitboard; 64] {
    let mut result = [Bitboard(0); 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut b = Bitboard(0);
            set_knight_dests(&mut b, x, y);
            result[(y * 8 + x) as usize].0 = b.0;
        }
    }
    result
}

fn make_ray_lookup(dx: i8, dy: i8) -> [Bitboard; 64] {
    let mut result = [Bitboard(0); 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut b = Bitboard(0);
            set_ray_dests(&mut b, x, y, dx, dy);
            result[(y * 8 + x) as usize].0 = b.0;
        }
    }
    result
}

fn set_knight_dests(b: &mut Bitboard, x: i8, y: i8) {

    b.slow_safe_set(x - 1, y + 2);
    b.slow_safe_set(x - 1, y - 2);

    b.slow_safe_set(x - 2, y + 1);
    b.slow_safe_set(x - 2, y - 1);

    b.slow_safe_set(x + 2, y + 1);
    b.slow_safe_set(x + 2, y - 1);

    b.slow_safe_set(x + 1, y + 2);
    b.slow_safe_set(x + 1, y - 2);
}

fn set_ray_dests(b: &mut Bitboard, x: i8, y: i8, dx: i8, dy: i8) {
    let mut cx = x;
    let mut cy = y;

    loop {
        cx += dx;
        cy += dy;

        if !b.slow_safe_set(cx, cy) { break; }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[ignore]
    #[test]
    fn eyeball_test() {
        let something = make_king_lookup();
        for i in 0..64 {
            println!("\n#{}\n{}", i, something[i]);
        }
    }
}
