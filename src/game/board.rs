use super::bitboard::{Bitboard, Square, is_empty, square_at, square_mask};
use super::hash::{PositionHash, pieces_key, white_to_move_key};
use super::moves::Move;
use super::pdn::{START_FEN, SidePieces, format_fen, parse_fen};
use super::piece::Piece;
use super::side::Side;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    black_men: Bitboard,
    black_kings: Bitboard,
    white_men: Bitboard,
    white_kings: Bitboard,
    side_to_move: Side,
    plies_since_progress: u8,
    hash: PositionHash,
}

impl Board {
    /// Create a board in the starting position, Black to move
    pub fn start_position() -> Self {
        Board::from_fen(START_FEN).expect("the starting position should always parse")
    }

    /// Create a board from a position in PDN FEN, like B:W21,22,K30:B1,2,K5
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parsed = parse_fen(fen)?;
        let mut board = Self {
            black_men: parsed.black.men,
            black_kings: parsed.black.kings,
            white_men: parsed.white.men,
            white_kings: parsed.white.kings,
            side_to_move: parsed.side_to_move,
            plies_since_progress: 0,
            hash: 0,
        };

        // Reject impossible positions
        if !board.is_valid() {
            return Err(format!("'{fen}' is not a valid position"));
        }

        // A new position computes its hash from scratch, after that each move updates it
        board.hash = board.compute_hash();

        Ok(board)
    }

    /// Write the position as a PDN FEN string
    pub fn to_fen(&self) -> String {
        let white = SidePieces {
            side: Side::White,
            men: self.white_men,
            kings: self.white_kings,
        };
        let black = SidePieces {
            side: Side::Black,
            men: self.black_men,
            kings: self.black_kings,
        };

        format_fen(self.side_to_move, &white, &black)
    }

    /// Return the hash that identifies this position
    pub fn hash(&self) -> PositionHash {
        self.hash
    }

    /// Compute the position hash from scratch
    fn compute_hash(&self) -> PositionHash {
        // Mix in the key for every piece on its square
        let mut hash = pieces_key(Piece::BlackMan, self.black_men)
            ^ pieces_key(Piece::BlackKing, self.black_kings)
            ^ pieces_key(Piece::WhiteMan, self.white_men)
            ^ pieces_key(Piece::WhiteKing, self.white_kings);

        // The same pieces with the other side to move is a different position
        if self.side_to_move == Side::White {
            hash ^= white_to_move_key();
        }

        hash
    }

    /// Return true if the position follows the basic rules of the board
    pub fn is_valid(&self) -> bool {
        // A square holds at most one piece, so the four bitboards can't overlap
        let piece_count: u32 = [
            self.black_men,
            self.black_kings,
            self.white_men,
            self.white_kings,
        ]
        .iter()
        .map(|bb| bb.count_ones())
        .sum();
        let occupied = !self.empty();
        let no_overlap = piece_count == occupied.count_ones();

        // A man that reached its promotion row would already be a king
        let black_crowned = is_empty(self.black_men & Side::Black.promotion_row());
        let white_crowned = is_empty(self.white_men & Side::White.promotion_row());

        no_overlap && black_crowned && white_crowned
    }

    /// Return the piece on the given square, or None if it's empty
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let bit = square_mask(square);

        if !is_empty(self.black_men & bit) {
            Some(Piece::BlackMan)
        } else if !is_empty(self.black_kings & bit) {
            Some(Piece::BlackKing)
        } else if !is_empty(self.white_men & bit) {
            Some(Piece::WhiteMan)
        } else if !is_empty(self.white_kings & bit) {
            Some(Piece::WhiteKing)
        } else {
            None
        }
    }

    /// Return the given side's men
    pub fn men(&self, side: Side) -> Bitboard {
        match side {
            Side::Black => self.black_men,
            Side::White => self.white_men,
        }
    }

    /// Return the given side's kings
    pub fn kings(&self, side: Side) -> Bitboard {
        match side {
            Side::Black => self.black_kings,
            Side::White => self.white_kings,
        }
    }

    /// Return all of the given side's pieces, men and kings
    pub fn pieces(&self, side: Side) -> Bitboard {
        self.men(side) | self.kings(side)
    }

    /// Return true if the given side has any pieces left
    pub fn has_pieces(&self, side: Side) -> bool {
        !is_empty(self.pieces(side))
    }

    /// Return every square with no piece on it
    pub fn empty(&self) -> Bitboard {
        !(self.pieces(Side::Black) | self.pieces(Side::White))
    }

    /// Return the side whose turn it is
    pub fn side_to_move(&self) -> Side {
        self.side_to_move
    }

    /// Return the plies played since a man moved or a piece was captured
    pub fn plies_since_progress(&self) -> u8 {
        self.plies_since_progress
    }

    /// Apply a legal move and pass the turn to the other side
    pub fn make_move(&mut self, mv: Move) {
        let side = self.side_to_move;
        let them = side.opposite();
        let from = square_mask(mv.from());
        let to = square_mask(mv.to());
        let captures = mv.captures();

        // Catch illegal moves before they corrupt the board
        debug_assert!(
            !is_empty(self.pieces(side) & from),
            "move {mv} starts on a square without one of our pieces"
        );
        debug_assert!(
            !is_empty((self.empty() | from) & to),
            "move {mv} lands on an occupied square"
        );
        debug_assert!(
            is_empty(captures & !self.pieces(them)),
            "move {mv} captures a square without one of their pieces"
        );

        // Figure out which of our pieces is moving
        let is_man = !is_empty(self.men(side) & from);
        let piece = if is_man {
            Piece::man(side)
        } else {
            Piece::king(side)
        };

        // A man that lands on our promotion row gets put down as a king
        let crowned = is_man && !is_empty(to & side.promotion_row());
        let placed = if crowned { Piece::king(side) } else { piece };

        // Pick the piece up and put it down on its new square
        self.remove_pieces(piece, from);
        self.place_pieces(placed, to);

        // Take every captured piece off the board
        self.remove_pieces(Piece::man(them), captures);
        self.remove_pieces(Piece::king(them), captures);

        self.update_progress(is_man, captures);
        self.switch_side();

        debug_assert!(self.is_valid(), "move {mv} left the board invalid");
        debug_assert_eq!(
            self.hash,
            self.compute_hash(),
            "move {mv} left the hash out of sync"
        );
    }

    /// Pass the turn to the other side
    fn switch_side(&mut self) {
        self.side_to_move = self.side_to_move.opposite();

        // The key is in the hash only while White is to move, so XOR toggles it
        self.hash ^= white_to_move_key();
    }

    /// Reset the 40-move count on progress, otherwise count one more ply
    fn update_progress(&mut self, is_man: bool, captures: Bitboard) {
        // Progress means a man moved or a piece was captured
        if is_man || !is_empty(captures) {
            self.plies_since_progress = 0;
        } else {
            self.plies_since_progress += 1;
        }
    }

    /// Put the given piece on every square in the mask
    fn place_pieces(&mut self, piece: Piece, mask: Bitboard) {
        let bitboard = self.bitboard_mut(piece);

        // Only squares that didn't already hold this piece change the hash
        let added = mask & !*bitboard;
        *bitboard |= mask;

        self.hash ^= pieces_key(piece, added);
    }

    /// Clear the given piece from every square in the mask
    fn remove_pieces(&mut self, piece: Piece, mask: Bitboard) {
        let bitboard = self.bitboard_mut(piece);

        // Only squares that actually held this piece change the hash
        let removed = mask & *bitboard;
        *bitboard &= !mask;

        self.hash ^= pieces_key(piece, removed);
    }

    /// Return the bitboard that stores the given piece
    fn bitboard_mut(&mut self, piece: Piece) -> &mut Bitboard {
        match piece {
            Piece::BlackMan => &mut self.black_men,
            Piece::BlackKing => &mut self.black_kings,
            Piece::WhiteMan => &mut self.white_men,
            Piece::WhiteKing => &mut self.white_kings,
        }
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
                let Some(square) = square_at(row, col) else {
                    write!(f, "{LIGHT_SQUARE_BG}   ")?;
                    continue;
                };

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Play the legal move with the given notation, like 9-13 or 22x31
    fn play(board: &mut Board, notation: &str) {
        let mv = Move::from_notation(board, notation).unwrap();
        board.make_move(mv);
    }

    /// Play a move from the given position and return the position after it
    fn after(fen: &str, notation: &str) -> String {
        let mut board = Board::from_fen(fen).unwrap();
        play(&mut board, notation);
        board.to_fen()
    }

    #[test]
    fn simple_step_moves_the_man_and_passes_the_turn() {
        assert_eq!(
            after(START_FEN, "9-13"),
            "W:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,10,11,12,13"
        );
    }

    #[test]
    fn capture_removes_the_jumped_piece() {
        assert_eq!(after("B:W14:B9", "9x18"), "W:W:B18");
    }

    #[test]
    fn multi_jump_removes_every_jumped_piece() {
        assert_eq!(after("B:W14,22:B9", "9x25"), "W:W:B25");
    }

    #[test]
    fn man_is_crowned_on_the_promotion_row() {
        // By a capture, and by a simple step
        assert_eq!(after("B:W26:B22", "22x31"), "W:W:BK31");
        assert_eq!(after("B:W:B27", "27-32"), "W:W:BK32");
    }

    #[test]
    fn king_stays_a_king() {
        assert_eq!(after("B:W:BK18", "18-14"), "W:W:BK14");
    }

    #[test]
    fn king_moves_count_toward_the_forty_move_rule() {
        let mut board = Board::from_fen("B:WK29:B1,K18").unwrap();

        play(&mut board, "18-14");
        assert_eq!(board.plies_since_progress(), 1);
        play(&mut board, "29-25");
        assert_eq!(board.plies_since_progress(), 2);

        // A man move is progress and resets the count
        play(&mut board, "1-5");
        assert_eq!(board.plies_since_progress(), 0);
    }

    #[test]
    fn a_capture_resets_the_forty_move_count() {
        let mut board = Board::from_fen("W:WK29,14:BK18").unwrap();

        play(&mut board, "29-25");
        assert_eq!(board.plies_since_progress(), 1);

        // Even a king capture counts as progress
        play(&mut board, "18x9");
        assert_eq!(board.plies_since_progress(), 0);
    }

    #[test]
    fn same_position_by_different_moves_has_the_same_hash() {
        let start = "B:WK29:BK1";

        // Black's king goes 1 -> 5 -> 9 in one game and 1 -> 6 -> 9 in the other
        let mut first = Board::from_fen(start).unwrap();
        for notation in ["1-5", "29-25", "5-9", "25-22"] {
            play(&mut first, notation);
        }

        let mut second = Board::from_fen(start).unwrap();
        for notation in ["1-6", "29-25", "6-9", "25-22"] {
            play(&mut second, notation);
        }

        assert_eq!(first.hash(), second.hash());
        assert_ne!(first.hash(), Board::from_fen(start).unwrap().hash());
    }

    #[test]
    fn side_to_move_changes_the_hash() {
        let black_to_move = Board::from_fen("B:W21:B1").unwrap();
        let white_to_move = Board::from_fen("W:W21:B1").unwrap();
        assert_ne!(black_to_move.hash(), white_to_move.hash());
    }
}
