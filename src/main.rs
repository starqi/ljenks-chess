mod board;

use board::{Player, Board, Square, Piece};
use std::{thread, time, io};

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
    board.set('d', 3, Square::Occupied(Piece::Knight, Player::Black));
    board.set('c', 5, Square::Occupied(Piece::Knight, Player::Black));
    println!("{}", format!("{:?}", board));

    loop {
        let mut x = String::new();
        io::stdin().read_line(&mut x).expect("?");

        if let Some(_file) = x.chars().nth(0) {
            if let Some(_rank) = x.chars().nth(1) {
                if let Some(__rank) = _rank.to_digit(10) {

                    let rank = __rank as u8;
                    println!("Parsed input: {}-{}", _file, rank);

                    println!("{:?}", board.get(_file, rank));

                    let mut r: Vec<(char, u8)> = Vec::new();
                    board.get_local_legal_moves(_file, rank, &mut r);
                    print_vec(&r);
                }
            }
        }
    }
}

fn print_vec<T : std::fmt::Debug>(v: &Vec<T>) {
    for item in v {
        print!("{:?} ", item);
    }
    println!();
}
