#[cfg(feature = "wasm")]
compile_error!("The CLI binary cannot be compiled with the wasm feature");

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

mod engine;
mod platform;
#[macro_use]
mod macros;

use engine::*;
use engine::init_globals;
use engine::ai::evaluation::evaluate;

fn main() {
    init_globals();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 || args[1] == "--help" || args[1] == "-h" {
        println!("Chess Engine CLI - Generate NNUE training data");
        println!();
        println!("Usage: {} <input_file> <output_file> [--depth N]", args[0]);
        println!();
        println!("Arguments:");
        println!("  input_file   File containing FEN strings (1 per line)");
        println!("  output_file  Binary output file for NNUE training data");
        println!("  --depth N    Search depth for evaluation (default: 7)");
        println!();
        println!("Output format:");
        println!("  Each position: [NNUE vector: 98324 bytes][score: 4 bytes]");
        println!();
        println!("Example:");
        println!("  {} positions.bin training_data.bin --depth 10", args[0]);
        std::process::exit(if args.len() < 3 { 1 } else { 0 });
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let depth: i8 = args.iter()
        .position(|x| x == "--depth")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);

    println!("Reading FENs from: {}", input_path);
    println!("Writing training data to: {}", output_path);
    println!("Evaluation depth: {}", depth);
    println!();

    let input_file = match File::open(input_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening input file: {}", e);
            std::process::exit(1);
        }
    };

    let output_file = match File::create(output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error creating output file: {}", e);
            std::process::exit(1);
        }
    };

    let reader: BufReader<File> = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);

    let mut total_positions = 0;
    let mut error_count = 0;
    let start_time = Instant::now();

    for (line_num, line) in reader.lines().enumerate() { // Rust note: `Lines` implements iterator, where Item = Result<String>
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line {}, skipping: {}", line_num + 1, e);
                error_count += 1;
                continue;
            }
        };

        let fen = line.trim();
        if fen.is_empty() {
            continue;
        }

        let mut board = match Board::from_fen(fen) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Invalid FEN at line {}, skipping: {} - {}", line_num + 1, fen, e);
                error_count += 1;
                continue;
            }
        };

        let mut nnue_vector = [0i8; Board::NNUE_TOTAL_SIZE];
        board.encode_nnue(&mut nnue_vector);

        // TODO IMMEDIATE CHANGE THIS WHOLE APPROACH DELETE BYTEMUCK
        // FIXME IMMEDIATE
        let score = evaluate(&board);

        // Now we have (vector, score) pair

        // TODO IMMEDIATE Minor Do we need bytemuck just for this?
        if let Err(e) = writer.write_all(bytemuck::cast_slice(&nnue_vector)) {
            eprintln!("Error writing NNUE vector for line {}, skipping: {}", line_num + 1, e);
            error_count += 1;
            continue;
            // TODO IMMEDIATE It will corrupt if errored?
        }

        if let Err(e) = writer.write_all(&score.to_le_bytes()) { // Little endian
            eprintln!("Error writing score for line {}, skipping: {}", line_num + 1, e);
            error_count += 1;
            continue;
        }

        total_positions += 1;

        if total_positions % 1000 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let rate = total_positions as f64 / elapsed;
            println!("Processed {} positions ({:.1} positions/sec)", total_positions, rate);
        }
    }

    if let Err(e) = writer.flush() {
        eprintln!("Error flushing output: {}", e);
        std::process::exit(1);
    }

    let elapsed = start_time.elapsed();
    println!();
    println!("Done!");
    println!("Total positions processed: {}", total_positions);
    println!("Errors: {}", error_count);
    println!("Time elapsed: {:.2}s", elapsed.as_secs_f64());
    if elapsed.as_secs() > 0 {
        println!("Average rate: {:.1} positions/sec", total_positions as f64 / elapsed.as_secs_f64());
    }
}
