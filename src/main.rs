mod board;
mod ai;

use ai::{Ai};
use board::{MoveList, CastleUtils, Board};
use std::{thread, io};
use std::sync::{Arc};

fn main() {

    env_logger::init();

    let mut board = Board::new();
    let castle_utils = CastleUtils::new();
    let mut ai = Ai::new();
    let mut y = String::new();

    /*
    let counter_ref = Arc::clone(&ai.counter);
    thread::spawn(move || {
        let duration = std::time::Duration::from_secs(5);
        loop {
            {
                let counter = counter_ref.lock().unwrap();
                println!("Evaluations = {}", counter);
            }
            thread::sleep(duration);
        }
    });
    */

    //let mut temp_moves = MoveList::new(10);
    //let mut moves_result = MoveList::new(10);

    loop {
        io::stdin().read_line(&mut y).expect("?");

        ai.make_move(&castle_utils, 3, &mut board);
        //board.get_moves(&castle_utils, &mut temp_moves, &mut moves_result);
    }
}

/*
fn main2() {

    env_logger::init();

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
    let mut temp_bufs = CheckThreatTempBuffers::new();

    loop {
        io::stdin().read_line(&mut y).expect("?");

        // TODO Design for this kind of borrow conflict?
        p_locs_v.clear();
        {
            let piece_locs = &board.get_player_state(board.get_player_with_turn()).piece_locs;
            piece_locs.iter().for_each(|(x, y)| {
                p_locs_v.push((*x, *y));
            });
        }

        let p_locs_i = rng.gen_range(0, p_locs_v.len());
        let (x, y) = p_locs_v[p_locs_i];
        let (file, rank) = match xy_to_file_rank_safe(x as i32, y as i32) {
            Ok(something) => something,
            Err(e) => {
                eprintln!("{:?}", e);
                break;
            }
        };

        println!("{:?}: Trying to move piece @ {}{}", board.get_player_with_turn(), file, rank);
        if let Err(e) = board.get_moves(file, rank, &mut temp_bufs, &mut move_list) {
            eprintln!("{:?}", e);
            break;
        }

        let xy_moves = move_list.get_moves();
        if xy_moves.len() > 0 {
            let move_i = rng.gen_range(0, xy_moves.len());
            let (target_x, target_y) = xy_moves[move_i];
            let (target_file, target_rank) = xy_to_file_rank_safe(target_x as i32, target_y as i32).unwrap();
            println!("... To {}{}", target_file, target_rank);
            if let Err(e) = board.make_move(&mut move_list, move_i) {
                eprintln!("{:?}", e);
                break;
            }
            println!("\n{}", board);
            debug_locations(&board);
        } else {
            println!("No moves...");
            continue;
        }
    }
}

// TODO Move to board print
fn debug_locations(board: &Board) {
    println!("\n{:?} pieces: ", board.get_player_with_turn());
    board
        .get_player_state(board.get_player_with_turn())
        .piece_locs
        .iter()
        .map(|(x, y)| xy_to_file_rank_safe(*x as i32, *y as i32))
        .filter(|r| r.is_ok())
        .for_each(|r| {
            let (file, rank) = r.unwrap();
            print!("{}{} ", file, rank);
        });
    println!("");
}
*/
