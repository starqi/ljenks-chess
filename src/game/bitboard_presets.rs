use super::bitboard::*;

pub enum RayDirection {
    Left = 0, LeftTop, Top, RightTop, Right, RightBottom, Bottom, LeftBottom
}

pub struct BitboardPresets {
    /// Index = `RayDirection` order
    pub rays: [[Bitboard; 64]; 8],
    pub knight_jumps: [Bitboard; 64],
    pub pawn_jumps: [[Bitboard; 64]; 2],
    pub pawn_pushes: [[Bitboard; 64]; 2],
    pub pawn_captures: [[Bitboard; 64]; 2],
    pub king_moves: [Bitboard; 64],
    pub perimeter: Bitboard
}

impl BitboardPresets {
    pub fn new() -> BitboardPresets {
        BitboardPresets {
            rays: [
                make_ray_lookup(-1, 0), make_ray_lookup(-1, -1), make_ray_lookup(0, -1), make_ray_lookup(1, -1),
                make_ray_lookup(1, 0), make_ray_lookup(1, 1), make_ray_lookup(0, 1), make_ray_lookup(-1, 1)
            ],
            knight_jumps: make_knight_jump_lookup(),
            pawn_jumps: [make_pawn2_lookup(-1, 6), make_pawn2_lookup(1, 1)],
            pawn_pushes: [make_pawn1_lookup(-1), make_pawn1_lookup(1)],
            pawn_captures: [make_pawn_capture_lookup(-1), make_pawn_capture_lookup(1)],
            king_moves: make_king_lookup(),
            perimeter: make_perimeter()
        }
    }
}

fn make_perimeter() -> Bitboard {
    let mut result = Bitboard(0);

    for y in [0, 7] {
        for x in 0..8 {
            result.set(x, y);
        }
    }

    for x in [0, 7] {
        for y in 1..7 {
            result.set(x, y);
        }
    }

    result
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

fn make_pawn2_lookup(dy: i8, jump_y: i8) -> [Bitboard; 64] {
    let mut result = [Bitboard(0); 64];
    for x in 0..8 {
        for y in 0..8 {
            if y == jump_y {
                let mut b = Bitboard(0);
                b.slow_safe_set(x, y + dy + dy);
                result[(y * 8 + x) as usize].0 = b.0;
            }
        }
    }
    result
}

fn make_pawn1_lookup(dy: i8) -> [Bitboard; 64] {
    let mut result = [Bitboard(0); 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut b = Bitboard(0);
            b.slow_safe_set(x, y + dy);
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
mod Test {

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
