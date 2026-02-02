use std::env;

mod engine;
mod platform;
#[macro_use]
mod macros;

use engine::*;
use engine::init_globals;

fn main() {
    init_globals();
    
    const DEFAULT_DEPTH: i8 = 7;
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("Chess Engine CLI - Evaluate chess positions");
        println!();
        println!("Usage: {} <FEN> [depth]", args[0]);
        println!();
        println!("Arguments:");
        println!("  FEN    FEN string representing the chess position");
        println!("  depth  Search depth (default: {})", DEFAULT_DEPTH);
        println!();
        println!("Example:");
        println!("  {} \"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1\" 10", args[0]);
        std::process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    let fen = &args[1];
    let depth: i8 = if args.len() >= 3 {
        args[2].parse().unwrap_or(DEFAULT_DEPTH)
    } else {
        DEFAULT_DEPTH
    };

    let mut board = match Board::from_fen(fen) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Invalid FEN: {}", e);
            std::process::exit(1);
        }
    };

    let position_hashes = vec![board.get_hash()];
    let half_moves_without_pawn_move = 0; // TODO Not implemented for FEN yet

    let mut ai = Ai::new();
    ai.late_inject(&position_hashes, &half_moves_without_pawn_move);

    println!("Evaluating position at depth {}...", depth);
    println!("FEN: {}", fen);
    
    // TODO IMMEDIATE Run it on a file of FENs
    let notation = ai.make_move(&mut board);
    if let Some(n) = notation {
        let score = ai.get_leading_move_with_score().map(|(_move, _depth, score)| score);
        if let Some(s) = score {
            println!("{} {}", n, s);
        } else {
            eprintln!("Unexpected notation with no score {}", n);
        }
    } else {
        println!("No move!");
    }
}
