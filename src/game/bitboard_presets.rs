use super::bitboard::*;

pub struct BitboardPresets {
    /// Index = LSB: left, left-top, top, right-top, then MSB: right, right-bottom, bottom, left-bottom.
    pub rays: [[Bitboard; 64]; 8],
    pub knight_jumps: [Bitboard; 64]
}

impl BitboardPresets {
    pub fn new() -> BitboardPresets {
        BitboardPresets {
            rays: [
                make_ray_lookup(-1, 0), make_ray_lookup(-1, -1), make_ray_lookup(0, -1), make_ray_lookup(1, -1),
                make_ray_lookup(1, 0), make_ray_lookup(1, 1), make_ray_lookup(0, 1), make_ray_lookup(-1, 1)
            ],
            knight_jumps: make_knight_jump_lookup()
        }
    }
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
    fn eyeball_test_knights() {
        let knights = make_knight_jump_lookup();
        for i in 0..64 {
            println!("Knight #{}\n{}", i, knights[i]);
        }
    }

    #[ignore]
    #[test]
    fn eyeball_test_left_top_ray() {
        let x = make_ray_lookup(-1, -1);
        for i in 0..64 {
            println!("Left-Top Ray #{}\n{}", i, x[i]);
        }
    }
}
