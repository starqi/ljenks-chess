mod board;

use rand::{thread_rng, Rng};
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

    let mut rng = thread_rng();

    let mut board = Board::new();
    println!("{}", board);
    debug_locations(&board);

    let mut move_list = MoveList::new();
    let mut p_locs_v: Vec<(u8, u8)> = Vec::new();
    let mut y = String::new();

    loop {
        io::stdin().read_line(&mut y).expect("?");

        let p_locs = board.get_piece_locations(board.player_with_turn);
        p_locs_v.splice(0.., p_locs.into_iter().map(|x| *x));
        let p_locs_i = rng.gen_range(0, p_locs_v.len());
        let (x, y) = p_locs_v[p_locs_i];
        let (file, rank) = Board::xy_to_file_rank(x, y);
        board.get_legal_moves(file, rank, &mut move_list);
        if move_list.get_moves().len() > 0 {
            let move_i = rng.gen_range(0, move_list.get_moves().len());
            board.make_move(&mut move_list, move_i);
            println!("{}", board);
            debug_locations(&board);
        } else {
            continue;
        }
    }
}

fn debug_locations(board: &Board) {
    board
        .get_piece_locations(board.player_with_turn)
        .into_iter()
        .map(|(x, y)| Board::xy_to_file_rank(*x, *y))
        .for_each(|x| print!("{:?}", x));
    println!("");
}

fn debug_iterator<T>(it: impl IntoIterator<Item = T>) where T : std::fmt::Debug {
    it.into_iter().for_each(|x| print!("{:?} ", x));
    println!("");
}
