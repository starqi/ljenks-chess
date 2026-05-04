#[cfg(feature = "wasm")]
compile_error!("The CLI binary cannot be compiled with the wasm feature");

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::Instant;
use clap::{Parser, Subcommand};

use ljenks_chess::{BestMoveInfoJs, Main};
use ljenks_chess::init_globals;
use ljenks_chess::Board;
use ljenks_chess::load_weights_safetensors;

#[derive(Parser, Debug)]
#[command(name = "chess-cli")]
#[command(about = "Chess Engine CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// TODO (Minor) Multiple ranges: 1-10,20-30

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate training positions into a .bin file
    Generate {
        /// Binary output file (appended, not overwritten)
        output_file: PathBuf,
        /// Random half moves at start
        #[arg(long, default_value = "20")]
        random_half_moves: usize,
        /// Node limit for search
        #[arg(long, default_value = "100000")]
        max_nodes: u64,
        /// Number of games to play
        #[arg(long, default_value = "1")]
        num_games: usize,
        #[arg(long, default_value = "300")]
        max_half_moves_per_game: Option<usize>,
    },
    /// View positions from a .bin file
    View {
        /// Binary data file with positions (.bin)
        file: PathBuf,
        /// Position range (e.g. "5", "1-10", "50-60", "all")
        range: String,
    },
    /// Evaluate positions with NNUE and print scores
    Nnue {
        /// Safetensors model file (.safetensors)
        model: PathBuf,
        /// Binary data file with positions (.bin)
        data: PathBuf,
        /// Position range (e.g. "5", "1-10", "50-60", "all")
        range: String,
    },
    /// Count positions in a .bin file
    Count {
        /// Binary data file with positions (.bin)
        file: PathBuf,
    },
}

fn parse_range(range: &str) -> (usize, usize) {
    if let Some((s, e)) = range.split_once('-') {
        let start = s.parse::<usize>().expect("Invalid range start").saturating_sub(1); // Rust: saturating_sub = No usize overflow
        let end = e.parse::<usize>().expect("Invalid range end");
        (start, end)
    } else if range == "all" {
        (0, usize::MAX)
    } else {
        let n = range.parse::<usize>().expect("Invalid range").saturating_sub(1);
        (n, n + 1)
    }
}

// [34, 38 magic numbers are from compressed.rs format]

struct PositionEntry {
    index: usize,
    board_bytes: [u8; 34],
    score: i32,
}

fn iterate_positions<F>(file_path: &PathBuf, range: &str, mut callback: F) -> io::Result<()>
    where F: FnMut(PositionEntry)
{
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; 38];
    let (start, end) = parse_range(range);
    let mut i = 0;
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read < 38 { break; }
        if i >= start && i < end {
            let board_copied: [u8; 34] = buffer[0..34].try_into().unwrap();
            let score_bytes = &buffer[34..38];
            let score = i32::from_le_bytes([score_bytes[0], score_bytes[1], score_bytes[2], score_bytes[3]]);
            callback(PositionEntry { index: i, board_bytes: board_copied, score });
        }
        i += 1;
        if i >= end { break; }
    }
    Ok(())
}

fn view_positions(file_path: &PathBuf, range: &str) -> io::Result<()> {
    iterate_positions(file_path, range, |entry| {
        let mut board = Board::with_kings_only();
        board.import_compressed(&entry.board_bytes);
        println!("Position {} - Score: {}", entry.index + 1, entry.score);
        println!("{}", board);
        println!();
    })
}

fn nnue_positions(model_path: &PathBuf, data_path: &PathBuf, range: &str) -> io::Result<()> {
    let model_bytes = std::fs::read(model_path)?;
    if !load_weights_safetensors(&model_bytes) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to load NNUE weights from safetensors file, see logs"));
    }
    iterate_positions(data_path, range, |entry| {
        let mut board = Board::with_kings_only();
        board.import_compressed(&entry.board_bytes);
        board.nnue_refresh_both();
        let nnue_score = board.nnue_forward();
        println!("Position {} - Target: {} - NNUE: {}", entry.index + 1, entry.score, nnue_score.map_or("N/A".to_string(), |s| s.to_string()));
    })
}

fn cmd_generate(
    output_file: PathBuf,
    random_half_moves: usize,
    max_nodes: u64,
    num_games: usize,
    max_half_moves_per_game: Option<usize>
) {
    let file: File = match File::create_new(&output_file) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            match File::options().append(true).open(&output_file) {
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

    let mut writer = BufWriter::new(file);
    let mut completed_games = 0;
    let mut total_positions = 0u64;
    let start_time = Instant::now();

    let mut main = Main::new();
    main.set_search_max_nodes(Some(max_nodes));
    let mut board_bytes = [0u8; 34];

    for _ in 0..num_games {
        main.new_board();

        let mut half_moves = 0;
        loop {
            if let Some(num) = main.get_game_end_state() { 
                println!("Game end state: {:?}", num);
                break; 
            }

            let random_early_moves = half_moves < random_half_moves;
            if let Some(m) = max_half_moves_per_game {
                if half_moves >= m {
                    println!("Exceeded {} max half moves, ending game", m);
                    break;
                }
            }
            main.get_board().export_compressed(&mut board_bytes);

            main.set_logging(false);
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
            main.set_logging(true);

            if move_result.is_none() {
                eprintln!("Unexpected no moves to make without formal game end {}", main.get_board());
                std::process::exit(1); // Exit, fix it
            }
            let move_result_unwrapped = move_result.unwrap();

            if let Err(e) = writer.write_all(&board_bytes) {
                eprintln!("Error writing board bytes: {}", e);
                std::process::exit(1);
            }
            if let Err(e) = writer.write_all(&move_result_unwrapped.score.to_le_bytes()) {
                eprintln!("Error writing score: {}", e);
                std::process::exit(1);
            }

            println!("\n{} {}", main.get_board(), move_result_unwrapped.score);
            half_moves += 1;
        }

        completed_games += 1;
        total_positions += half_moves as u64;
        println!("Half moves: {}, now completed {} games", half_moves, completed_games);
        
        if let Err(e) = writer.flush() {
            eprintln!("Error flushing output: {}", e);
            std::process::exit(1);
        }
    }

    let elapsed = start_time.elapsed();
    println!();
    println!("Done!");
    println!("Games completed: {}", completed_games);
    println!("Positions: {}", total_positions);
    let secs = elapsed.as_secs_f64();
    println!("Time elapsed: {:.2}s", secs);
    if secs > 0.0 {
        println!("Positions/sec: {:.0}", total_positions as f64 / secs);
    }
}

fn main() {
    init_globals();
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output_file, random_half_moves, max_nodes, num_games, max_half_moves_per_game } => {
            cmd_generate(output_file, random_half_moves, max_nodes, num_games, max_half_moves_per_game);
        }
        Commands::View { file, range } => {
            if let Err(e) = view_positions(&file, &range) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Nnue { model, data, range } => {
            if let Err(e) = nnue_positions(&model, &data, &range) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Count { file } => {
            let metadata = std::fs::metadata(&file).expect("Failed to read file metadata");
            if metadata.len() % 38 != 0 {
                eprintln!("Warning: file size {} is not a multiple of 38 bytes", metadata.len());
            }
            println!("{}", metadata.len() / 38);
        }
    }
}
