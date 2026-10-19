use rand::seq::IndexedRandom;

use super::limits::SearchLimits;
use crate::game::{Board, Move, PositionHash, generate_moves};
use std::sync::atomic::AtomicBool;

/// Return the move to play in the position, or None if there are no legal moves.
/// The history is every earlier position's hash, oldest first, for spotting repetitions
pub fn search(
    board: &Board,
    _history: &[PositionHash],
    _limits: SearchLimits,
    _stop: &AtomicBool,
) -> Option<Move> {
    let moves = generate_moves(board);
    let mut rng = rand::rng();
    moves.choose(&mut rng).copied()
}
