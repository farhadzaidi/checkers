use super::types::{Bitboard, Square};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    from: Square,
    to: Square,
    captures: Bitboard,
}

impl Move {
    /// Create a move; `captures` holds every jumped piece, or 0 for a simple step
    pub fn new(from: Square, to: Square, captures: Bitboard) -> Self {
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
}
