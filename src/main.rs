use rand::Rng;
use std::cmp::Ordering;
use std::{thread, time, io};

fn old_main() {
    let n = rand::thread_rng().gen_range(1, 105);

    loop {

        let mut x = String::new();
        io::stdin().read_line(&mut x).expect("?");
        //println!("x: {}", x);

        let x2: i32 = match x.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("NaN");
                continue;
            }
        };

        match x2.cmp(&n) {
            Ordering::Less => println!("<"),
            Ordering::Greater => println!(">"),
            Ordering::Equal => {
                println!("=");
                break;
            }
        };
    }
}

fn main() {
    /*
    thread::spawn(move || {
        let duration = std::time::Duration::from_secs(2);
        loop {
            println!("Hurry up");
            thread::sleep(duration);
        }
    });

    old_main();
    */

    let mut board = Board::new();
    println!("{}", format!("{:?}", board));
}

#[derive(Copy, Clone, Debug)]
enum Piece {
    Pawn, Rook, Knight, Bishop, Queen, King, Blank
}

#[derive(Debug)]
struct Board {
    arr: [Piece; 64]
}

impl Board {
    fn new() -> Board {
        let mut board = Board {
            arr: [Piece::Blank; 64]
        };
        board.set_pawn_row(1);
        board.set_pawn_row(8);

        board.set_main_row(1);
        board.set_main_row(8);

        board
    }

    fn set_pawn_row(&mut self, rank: u8) {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank as usize, Piece::Pawn);
        }
    }

    fn set_main_row(&mut self, rank: u8) {
        self.set('a', rank, Piece::Rook);
        self.set('b', rank, Piece::Knight);
        self.set('c', rank, Piece::Bishop);
        self.set('d', rank, Piece::Queen);
        self.set('e', rank, Piece::King);
        self.set('f', rank, Piece::Bishop);
        self.set('g', rank, Piece::Knight);
        self.set('h', rank, Piece::Rook);
    }

    fn get_by_xy(&self, x: usize, y: usize) -> Piece {
        return self.arr[y * 8 + x];
    }

    fn set_by_xy(&mut self, x: usize, y: usize, p: Piece) {
        self.arr[y * 8 + x] = p;
    }

    fn set(&mut self, file: char, rank: u8, p: Piece) {
        let x = file as u32 - 'a' as u32;
        let y = 8 - rank;
        self.set_by_xy(x as usize, y as usize, p);
    }
}
