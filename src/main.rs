mod board;

use board::{Player, Board, Square, Piece, MoveList};
use std::{thread, io};

fn main() {

    thread::spawn(move || {
        let duration = std::time::Duration::from_secs(5);
        loop {
            //println!("Hurry up");
            thread::sleep(duration);
        }
    });

    let mut board = Board::new();
    //board.set('d', 3, Square::Occupied(Piece::Knight, Player::Black));
    //board.set('c', 5, Square::Occupied(Piece::Knight, Player::Black));
    //board.set('a', 2, Square::Blank);
    //board.set('b', 1, Square::Blank);
    //board.set('h', 7, Square::Blank);
    println!("{}", format!("{:?}", board));

    let mut move_list = MoveList::new();

    loop {
        let mut x = String::new();
        io::stdin().read_line(&mut x).expect("?");


        if let Some(_file) = x.chars().nth(0) {
            if let Some(_rank) = x.chars().nth(1) {
                if let Some(__rank) = _rank.to_digit(10) {

                    // TODO Extract parsing as helper

                    let rank = __rank as u8;
                    println!("Parsed input: {}-{}", _file, rank);
                    println!("{:?}", board.get(_file, rank));

                    board.get_legal_moves(_file, rank, &mut move_list);
                    println!("Moves:");
                    print_vec(move_list.get_moves());

                    let mut y = String::new();
                    io::stdin().read_line(&mut y).expect("?");
                    match y.trim().parse::<usize>() {
                        Ok(i) => {
                            println!("Moving");
                            board.make_move(&mut move_list, i);
                        },
                        Err(e) => {
                            println!("No move, {}", e);
                            continue;
                        }
                    };
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
