use crate::game::{Board, generate_moves};
use std::time::Instant;

/// Count every position exactly the given number of plies ahead
pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_moves(board);

    // With one ply left every move is a leaf, so count them without playing them
    if depth == 1 {
        return moves.len() as u64;
    }

    // Play each move on a copy of the board and count what's below it
    let mut nodes = 0;
    for &mv in &moves {
        let mut child = *board;
        child.make_move(mv);
        nodes += perft(&child, depth - 1);
    }

    nodes
}

/// Print the node count and speed for every depth up to the given one
pub fn run(board: &Board, max_depth: u32) {
    for depth in 1..=max_depth {
        let start = Instant::now();
        let nodes = perft(board, depth);
        let seconds = start.elapsed().as_secs_f64();

        // Guard against dividing by zero on the tiny depths
        let nps = nodes as f64 / seconds.max(1e-9);
        println!("depth {depth:>2}: {nodes:>14} nodes  {seconds:>8.3}s  {nps:>14.0} nps");
    }
}

/// Print the node count under each root move, then the total
pub fn divide(board: &Board, depth: u32) {
    let mut total = 0;

    for &mv in &generate_moves(board) {
        let mut child = *board;
        child.make_move(mv);
        let nodes = perft(&child, depth - 1);

        println!("{mv}: {nodes}");
        total += nodes;
    }

    println!("\ntotal: {total}");
}
