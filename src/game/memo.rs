use super::super::extern_funcs::random;
use super::super::*;

fn get_random_u64() -> u64 {
    let mut a = ((u16::MAX as f64) * random()) as u64;

    a <<= 16;
    a += ((u16::MAX as f64) * random()) as u64;

    a <<= 16;
    a += ((u16::MAX as f64) * random()) as u64;

    a <<= 16;
    a += ((u16::MAX as f64) * random()) as u64;
    
    a
}

pub const PIECE_LEN: usize = 6;
pub const PER_SQUARE_LEN: usize = PIECE_LEN * 2;
pub const SQUARES_LEN: usize = 64 * PER_SQUARE_LEN; 

pub struct RandomNumberKeys {
    pub squares: [u64; SQUARES_LEN],
    /// First index: `CastleType` enum number. Second index: `Player` enum number.
    pub moved_castle_piece: [[u64; 2]; 2],
    pub is_white_to_play: u64
}

impl RandomNumberKeys {
    pub fn new() -> RandomNumberKeys {
        console_log!("Generating random number keys for hashing");
        let mut squares = [0u64; SQUARES_LEN];
        for i in 0..squares.len() {
            squares[i] = get_random_u64();
        }
        RandomNumberKeys {
            squares,
            moved_castle_piece: [[get_random_u64(), get_random_u64()], [get_random_u64(), get_random_u64()]],
            is_white_to_play: get_random_u64()
        }
    }
}
