use super::bitboard::{Bitboard, Square, is_empty};
use arrayvec::ArrayVec;
use std::fmt;

/// Upper bound on legal moves in one position, with plenty of headroom
pub const MAX_MOVES: usize = 128;

/// Fixed-capacity move list stored on the stack
pub type MoveList = ArrayVec<Move, MAX_MOVES>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    from: Square,
    to: Square,
    captures: Bitboard,
}

impl Move {
    /// Create a move from one square to another, capturing the given pieces
    pub fn new(from: Square, to: Square, captures: Bitboard) -> Self {
        debug_assert!(from < 32 && to < 32, "move {from}->{to} is off the board");

        // Only a capture can end where it started (a king looping back)
        debug_assert!(
            from != to || !is_empty(captures),
            "a simple step must change squares"
        );

        Self { from, to, captures }
    }

    /// Return the square the piece starts on
    pub fn from(&self) -> Square {
        self.from
    }

    /// Return the square the piece ends on
    pub fn to(&self) -> Square {
        self.to
    }

    /// Return the pieces this move captures
    pub fn captures(&self) -> Bitboard {
        self.captures
    }

    /// Return true if this move jumps over at least one piece
    pub fn is_capture(&self) -> bool {
        !is_empty(self.captures)
    }
}

impl fmt::Display for Move {
    /// Write the move in standard notation, like 11-15 or 15x24
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Notation numbers squares 1-32, but we store them as bits 0-31
        let from = self.from + 1;
        let to = self.to + 1;

        // Captures use an x, simple steps use a dash
        let separator = if self.is_capture() { 'x' } else { '-' };
        write!(f, "{from}{separator}{to}")
    }
}
