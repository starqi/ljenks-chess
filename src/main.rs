#[cfg(feature = "wasm")]
compile_error!("The CLI binary cannot be compiled with the wasm feature");

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::Instant;
use clap::Parser;

use ljenks_chess::{BestMoveInfoJs, Main};
use ljenks_chess::init_globals;
use ljenks_chess::Board;

#[derive(Parser, Debug)]
#[command(name = "chess-cli")]
#[command(about = "Chess Engine CLI - NNUE Training Data Generator", long_about = None)]
struct Cli {
    /// Binary output file (appended, not overwritten)
    output_file: PathBuf,
    
    /// Random half moves at start
    #[arg(long, default_value = "10")]
    random_half_moves: usize,
    
    /// Node limit for search
    #[arg(long, default_value = "300000")]
    max_nodes: u64,
    
    /// Number of games to play
    #[arg(long, default_value = "2")]
    num_games: usize,

    #[arg(long, default_value = "100")]
    max_half_moves_per_game: Option<usize>,
    
    /// View positions from file (e.g. "10", "5-15", "all")
    #[arg(long)]
    view: Option<String>,
}

fn view_positions(file_path: &PathBuf, range: &str) -> io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; 38];
    
    let (start, end) = if let Some((s, e)) = range.split_once('-') {
        let start = s.parse::<usize>().unwrap_or(1).saturating_sub(1);
        let end = e.parse::<usize>().unwrap_or(usize::MAX);
        (start, end)
    } else if range == "all" {
        (0, usize::MAX)
    } else {
        let n = range.parse::<usize>().unwrap_or(10);
        (0, n)
    };

    let mut i = 0;
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read < 38 { break; }
        
        if i >= start && i < end {
            let board_copied: [u8; 34] = buffer[0..34].try_into().unwrap();
            let score_bytes = &buffer[34..38];
            let score = i32::from_le_bytes([score_bytes[0], score_bytes[1], score_bytes[2], score_bytes[3]]);
            
            let mut board = Board::with_kings_only();
            board.import_compressed(&board_copied);
            
            println!("Position {} - Score: {}", i + 1, score);
            println!("{}", board);
            println!();
        }
        
        i += 1;
        if i >= end { break; }
    }
    
    Ok(())
}

fn main() {
    init_globals();
    
    let cli = Cli::parse();
    
    if let Some(range) = &cli.view {
        if let Err(e) = view_positions(&cli.output_file, range) {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let output_file: File = match File::create_new(&cli.output_file) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            match File::options().append(true).open(&cli.output_file) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error opening existing output file: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("Error creating output file: {}", e);
            std::process::exit(1);
        }
    };

    let mut writer = BufWriter::new(output_file);
    let mut completed_games = 0;
    let start_time = Instant::now();

    let mut main = Main::new();
    main.set_search_max_nodes(Some(cli.max_nodes));
    let mut board_bytes = [0u8; 34];

    for _ in 0..cli.num_games {
        main.new_board();
        
        let mut half_moves = 0;
        loop {
            if main.get_game_end_state().is_some() { break; }
            let random_early_moves = half_moves < cli.random_half_moves;
            if let Some(m) = cli.max_half_moves_per_game {
                if half_moves >= m {
                    println!("Exceeded {} max half moves, ending game", m);
                    break;
                }
            }
            main.get_board().export_compressed(&mut board_bytes);
            let move_result: Option<BestMoveInfoJs> = if random_early_moves {
                let score = main.evaluate().map(|info| info.score).expect("Unexpected game ended while evaluating");
                let mut r = main.make_random_move();
                if let Some(rr) = &mut r {
                    (*rr).score = score;
                }
                r
            } else {
                main.make_ai_move()
            };
            if move_result.is_none() {
                eprintln!("Unexpected no moves to make without formal game end {}", main.get_board());
                std::process::exit(1); // Exit, fix it
            }

            if let Err(e) = writer.write_all(&board_bytes) {
                eprintln!("Error writing board bytes: {}", e);
                std::process::exit(1);
            }
            if let Err(e) = writer.write_all(&move_result.unwrap().score.to_le_bytes()) {
                eprintln!("Error writing score: {}", e);
                std::process::exit(1);
            }

            half_moves += 1;
        }

        if let Some(end_state) = main.get_game_end_state() {
            println!("End game state: {}", end_state);
        }

        completed_games += 1;
        if completed_games % 3 == 0 {
            println!("Completed {} games", completed_games);
        }
        
        if let Err(e) = writer.flush() {
            eprintln!("Error flushing output: {}", e);
            std::process::exit(1);
        }
    }

    let elapsed = start_time.elapsed();
    println!();
    println!("Done!");
    println!("Games completed: {}", completed_games);
    println!("Time elapsed: {:.2}s", elapsed.as_secs_f64());
}
