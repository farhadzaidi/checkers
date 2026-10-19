use super::bitboard::{Bitboard, Square, is_empty, pop_lsb};
use super::board::Board;
use super::movegen::{generate_moves, move_path};
use super::pdn::{describes, format_move, parse_move};
use arrayvec::ArrayVec;
use std::fmt;

/// Upper bound on legal moves in one position, with plenty of headroom
pub const MAX_MOVES: usize = 128;

/// Fixed-capacity move list stored on the stack
pub type MoveList = ArrayVec<Move, MAX_MOVES>;

/// Most pieces one move can capture: every piece the other side starts with
pub const MAX_CAPTURES: usize = 12;

/// Squares captured by one move, stored on the stack
pub type CapturedSquares = ArrayVec<Square, MAX_CAPTURES>;

/// Squares a move lands on in order, ending on its final square. One per jump, or one for a simple step
pub type MovePath = ArrayVec<Square, MAX_CAPTURES>;

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

    /// Return the squares of the pieces this move captures
    pub fn captured_squares(&self) -> CapturedSquares {
        // Take the captured pieces off one at a time, lowest square first
        //     captures on 9 and 18 -> [9, 18]
        let mut squares = CapturedSquares::new();
        let mut remaining = self.captures;
        while !is_empty(remaining) {
            squares.push(pop_lsb(&mut remaining));
        }
        squares
    }

    /// Return the legal move in the position that the notation describes, like 9-13, 15x24 or 6x13x22
    pub fn from_notation(board: &Board, notation: &str) -> Result<Move, String> {
        let parsed = parse_move(notation)?;

        // Keep the legal moves the notation describes. Two moves can be exactly alike (a king
        // looping around its square either way), so each is kept once
        let mut matches = MoveList::new();
        for mv in generate_moves(board) {
            let path = move_path(board, mv);
            if describes(mv, &parsed, &path) && !matches.contains(&mv) {
                matches.push(mv);
            }
        }

        match matches.as_slice() {
            [mv] => Ok(*mv),
            [] => Err(format!(
                "{notation} is not a legal move in {}",
                board.to_fen()
            )),
            _ => Err(format!(
                "{notation} could be more than one capture; write every square it lands on, like 6x13x22"
            )),
        }
    }
}

impl fmt::Display for Move {
    /// Write the move in standard notation, like 11-15 or 15x24
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format_move(*self))
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{Board, Move};

    // Black's man on 6 can capture around a diamond of White men on 9, 10, 17 and 18,
    // landing on 22 either way: through 13 or through 15
    const DIAMOND: &str = "B:W9,10,17,18:B6";

    /// Return the legal move with the given notation in a position
    fn find_move(fen: &str, notation: &str) -> Result<Move, String> {
        Move::from_notation(&Board::from_fen(fen).unwrap(), notation)
    }

    #[test]
    fn double_jump_lists_both_captured_squares() {
        // Black jumps 18 then 27, which are squares 17 and 26 counting from 0
        let mv = find_move("B:W18,27:B14", "14x32").unwrap();
        assert_eq!(mv.captured_squares().as_slice(), &[17, 26]);
    }

    #[test]
    fn simple_step_captures_nothing() {
        let mv = find_move("B:W18:B9", "9-13").unwrap();
        assert!(mv.captured_squares().is_empty());
    }

    #[test]
    fn notation_reads_back_what_display_writes() {
        let mv = find_move("B:W14,22:B9", "9x25").unwrap();
        assert_eq!(mv.to_string(), "9x25");
    }

    #[test]
    fn short_notation_for_two_captures_is_ambiguous() {
        assert!(find_move(DIAMOND, "6x22").is_err());
    }

    #[test]
    fn full_route_picks_one_of_two_captures() {
        // Through 13 takes 9 and 17; through 15 takes 10 and 18 (squares 8, 16 and 9, 17 counting from 0)
        let left = find_move(DIAMOND, "6x13x22").unwrap();
        let right = find_move(DIAMOND, "6x15x22").unwrap();
        assert_eq!(left.captured_squares().as_slice(), &[8, 16]);
        assert_eq!(right.captured_squares().as_slice(), &[9, 17]);
    }

    #[test]
    fn identical_king_loops_count_as_one_move() {
        // Going around 10 either way is the same move, so it isn't ambiguous
        assert!(find_move("B:W14,15,22,23:BK10", "10x10").is_ok());
    }

    #[test]
    fn illegal_moves_are_rejected() {
        // A real square but not a legal move, and a step written where a capture is forced
        assert!(find_move(DIAMOND, "6-11").is_err());
        assert!(find_move("B:W14:B1,9", "1-5").is_err());
    }
}
