use super::constants::{START_BLACK_KINGS, START_BLACK_MEN, START_WHITE_KINGS, START_WHITE_MEN};
use super::piece::Piece;
use super::side::Side;
use super::types::{Bitboard, Square};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    black_men: Bitboard,
    black_kings: Bitboard,
    white_men: Bitboard,
    white_kings: Bitboard,
    side_to_move: Side,
}

impl Board {
    /// Create a board in the starting position, Black to move
    pub fn new() -> Self {
        Self {
            black_men: START_BLACK_MEN,
            black_kings: START_BLACK_KINGS,
            white_men: START_WHITE_MEN,
            white_kings: START_WHITE_KINGS,
            side_to_move: Side::Black,
        }
    }

    /// Return the piece on `square`, or None if it's empty
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let bit: Bitboard = 1 << square;

        if self.black_men & bit != 0 {
            Some(Piece::BlackMan)
        } else if self.black_kings & bit != 0 {
            Some(Piece::BlackKing)
        } else if self.white_men & bit != 0 {
            Some(Piece::WhiteMan)
        } else if self.white_kings & bit != 0 {
            Some(Piece::WhiteKing)
        } else {
            None
        }
    }

    /// Return `side`'s men
    pub fn men(&self, side: Side) -> Bitboard {
        match side {
            Side::Black => self.black_men,
            Side::White => self.white_men,
        }
    }

    /// Return `side`'s kings
    pub fn kings(&self, side: Side) -> Bitboard {
        match side {
            Side::Black => self.black_kings,
            Side::White => self.white_kings,
        }
    }

    /// Return all of `side`'s pieces, men and kings
    pub fn pieces(&self, side: Side) -> Bitboard {
        self.men(side) | self.kings(side)
    }

    /// Return every square with no piece on it
    pub fn empty_squares(&self) -> Bitboard {
        !(self.pieces(Side::Black) | self.pieces(Side::White))
    }

    /// Return the side whose turn it is
    pub fn side_to_move(&self) -> Side {
        self.side_to_move
    }
}

// ANSI 256-color escape codes for the terminal board
const RESET: &str = "\x1b[0m";
const LIGHT_SQUARE_BG: &str = "\x1b[48;5;250m";
const DARK_SQUARE_BG: &str = "\x1b[48;5;60m";
const BLACK_PIECE_FG: &str = "\x1b[38;5;235m";
const WHITE_PIECE_FG: &str = "\x1b[38;5;254m";

const MAN_SYMBOL: char = '●';
const KING_SYMBOL: char = '◉';

impl fmt::Display for Board {
    /// Draw the 8x8 board with Black at the top:
    ///   light grey squares = light (never used)
    ///   slate squares = dark (playable)
    ///   ● = man, ◉ = king, colored black or white by side
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..8 {
            for col in 0..8 {
                // Light squares are never used
                if (row + col) % 2 == 0 {
                    write!(f, "{LIGHT_SQUARE_BG}   ")?;
                    continue;
                }

                // Each row holds 4 dark squares, one per column pair
                let square = row * 4 + col / 2;
                let (color, symbol) = match self.piece_at(square) {
                    Some(Piece::BlackMan) => (BLACK_PIECE_FG, MAN_SYMBOL),
                    Some(Piece::BlackKing) => (BLACK_PIECE_FG, KING_SYMBOL),
                    Some(Piece::WhiteMan) => (WHITE_PIECE_FG, MAN_SYMBOL),
                    Some(Piece::WhiteKing) => (WHITE_PIECE_FG, KING_SYMBOL),
                    None => ("", ' '),
                };
                write!(f, "{DARK_SQUARE_BG}{color} {symbol} ")?;
            }
            writeln!(f, "{RESET}")?;
        }

        write!(f, "{:?} to move", self.side_to_move)
    }
}
