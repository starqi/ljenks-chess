use std::env;
use std::time::Instant;

mod game;
mod ai;
mod macros;

#[cfg(feature = "cli")]
mod extern_funcs;

use ai::*;
use game::bitboard_presets::*;
use game::memo::*;
use game::coords::*;
use game::entities::*;
use game::board::*;
use game::castle_utils::*;
use game::searchable_moves::*;
use game::move_list::*;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref CASTLE_UTILS: CastleUtils = CastleUtils::new();
    pub static ref RANDOM_NUMBER_KEYS: RandomNumberKeys = RandomNumberKeys::new();
    pub static ref BITBOARD_PRESETS: BitboardPresets = BitboardPresets::new();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("Chess Engine CLI - Evaluate chess positions");
        println!();
        println!("Usage: {} <FEN> [depth]", args[0]);
        println!();
        println!("Arguments:");
        println!("  FEN    FEN string representing the chess position");
        println!("  depth  Search depth (default: 10)");
        println!();
        println!("Example:");
        println!("  {} \"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1\" 10", args[0]);
        std::process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    let fen = &args[1];
    let depth: i8 = if args.len() >= 3 {
        args[2].parse().unwrap_or(10)
    } else {
        10
    };

    let start = Instant::now();
    
    let board = match Board::from_fen(fen) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Invalid FEN: {}", e);
            std::process::exit(1);
        }
    };

    let position_hashes = vec![board.get_hash()];
    let half_moves_without_pawn_move = 0;

    let mut ai = Ai::new();
    ai.late_inject(&position_hashes, &half_moves_without_pawn_move);

    println!("Evaluating position at depth {}...", depth);
    println!("FEN: {}", fen);
    
    let config = SearchConfig {
        depth,
        time_limit_ms: None,
    };
    
    let result = ai.evaluate_position(&board, config);
    
    println!("\nEvaluation: {}", result.score);
    println!("Nodes searched: {}", result.nodes_searched);
    println!("Time: {:.3}s", result.time_ms as f64 / 1000.0);
    println!("NPS: {:.0}", result.nps);
    
    if let Some(best_move) = result.best_move {
        println!("Best move: {}", best_move);
    }
}