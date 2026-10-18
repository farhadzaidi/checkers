use checkers::game::Board;
use checkers::perft;
use std::env;

/// Run perft or divide from the starting position
fn main() {
    let args: Vec<String> = env::args().collect();
    let board = Board::start_position();

    // Usage: cargo run --release -- perft <depth>
    //        cargo run --release -- divide <depth>
    match args.get(1).map(String::as_str) {
        Some("perft") => perft::run(&board, parse_depth(&args)),
        Some("divide") => perft::divide(&board, parse_depth(&args)),
        _ => eprintln!("usage: perft <depth> | divide <depth>"),
    }
}

/// Read the depth that follows a perft or divide command
fn parse_depth(args: &[String]) -> u32 {
    args.get(2)
        .and_then(|depth| depth.parse().ok())
        .filter(|&depth| depth > 0)
        .expect("expected a depth of at least 1, like: perft 8")
}