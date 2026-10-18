use super::bitboard::{Bitboard, EMPTY_BITBOARD, is_empty, pop_lsb, square_mask};
use super::side::Side;
use super::types::Square;

// Positions are written in PDN's FEN format, three fields split by colons:
//     1. Side to move: B or W
//     2. White's pieces: W, then its squares
//     3. Black's pieces: B, then its squares
// Squares are numbered 1-32 and separated by commas. A K marks a king.
//
// Example: B:W21,22,K30:B1,2,K5
//     Black to move
//     White: men on 21 and 22, king on 30
//     Black: men on 1 and 2, king on 5

/// The standard starting position, Black to move
pub const START_FEN: &str =
    "B:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,9,10,11,12";

/// The side to move and pieces read from a FEN string, before they become a board
pub struct ParsedPosition {
    pub side_to_move: Side,
    pub white: SidePieces,
    pub black: SidePieces,
}

/// One side's men and kings as bitboards
pub struct SidePieces {
    pub side: Side,
    pub men: Bitboard,
    pub kings: Bitboard,
}

/// Parse a PDN FEN string into its side to move and piece lists
pub fn parse_fen(fen: &str) -> Result<ParsedPosition, String> {
    // Split into the three fields
    //     B:W21,22,K30:B1,2,K5 -> "B", "W21,22,K30", "B1,2,K5"
    let fields: Vec<&str> = fen.trim().split(':').collect();
    if fields.len() != 3 {
        return Err(format!("expected 3 fields split by ':', found {}", fields.len()));
    }

    let side_to_move = parse_side(fields[0])?;
    let white = parse_pieces(fields[1])?;
    let black = parse_pieces(fields[2])?;

    // The piece lists must be White's then Black's
    if white.side != Side::White || black.side != Side::Black {
        return Err("expected White's pieces, then Black's".to_string());
    }

    Ok(ParsedPosition {
        side_to_move,
        white,
        black,
    })
}

/// Parse a piece list like W21,22,K30
fn parse_pieces(text: &str) -> Result<SidePieces, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("found an empty piece list".to_string());
    }

    // The first character is the side, the rest is the squares
    //     W21,22,K30 -> "W" and "21,22,K30"
    let (letter, squares) = text.split_at(1);
    let side = parse_side(letter)?;

    let mut men = EMPTY_BITBOARD;
    let mut kings = EMPTY_BITBOARD;

    // Go through the squares one at a time
    //     "21,22,K30" -> "21", "22", "K30"
    for entry in squares.split(',') {
        // A side with no pieces is just its letter, which leaves one empty entry
        if entry.trim().is_empty() {
            continue;
        }

        let (is_king, square) = parse_entry(entry)?;
        let mask = square_mask(square);

        // Each square can only be listed once
        if !is_empty((men | kings) & mask) {
            return Err(format!("square {} is listed twice", square + 1));
        }

        if is_king {
            kings |= mask;
        } else {
            men |= mask;
        }
    }

    Ok(SidePieces { side, men, kings })
}

/// Parse one entry of a piece list into whether it's a king and its square
///     "21"  -> (false, square 21)
///     "K30" -> (true, square 30)
fn parse_entry(entry: &str) -> Result<(bool, Square), String> {
    let entry = entry.trim();
    match entry.strip_prefix('K') {
        Some(number) => Ok((true, parse_square(number)?)),
        None => Ok((false, parse_square(entry)?)),
    }
}

/// Parse a side letter, B or W
fn parse_side(text: &str) -> Result<Side, String> {
    match text.trim() {
        "B" => Ok(Side::Black),
        "W" => Ok(Side::White),
        other => Err(format!("expected side B or W, found '{other}'")),
    }
}

/// Parse a square number from 1 to 32 into its bit index
fn parse_square(text: &str) -> Result<Square, String> {
    match text.trim().parse::<Square>() {
        Ok(number @ 1..=32) => Ok(number - 1),
        _ => Err(format!("expected a square from 1 to 32, found '{text}'")),
    }
}

/// Write a position as a PDN FEN string
pub fn format_fen(side_to_move: Side, white: &SidePieces, black: &SidePieces) -> String {
    // Write each field, then join them with colons
    //     "B", "W21,22,K30", "B1,2,K5" -> B:W21,22,K30:B1,2,K5
    let turn = side_letter(side_to_move);
    let white = format_pieces(white);
    let black = format_pieces(black);
    format!("{turn}:{white}:{black}")
}

/// Write a piece list like W21,22,K30
fn format_pieces(pieces: &SidePieces) -> String {
    let mut entries = Vec::new();

    // Walk the squares in order, so men and kings stay sorted together
    let mut remaining = pieces.men | pieces.kings;
    while !is_empty(remaining) {
        let square = pop_lsb(&mut remaining);
        entries.push(format_entry(square, pieces.kings));
    }

    // The side letter, then the squares split by commas
    //     "W" and "21", "22", "K30" -> W21,22,K30
    format!("{}{}", side_letter(pieces.side), entries.join(","))
}

/// Write one square of a piece list, with a K in front if it holds a king
///     square 21, man  -> "21"
///     square 30, king -> "K30"
fn format_entry(square: Square, kings: Bitboard) -> String {
    // Squares are bits 0-31, but notation numbers them 1-32
    let number = square + 1;

    if is_empty(kings & square_mask(square)) {
        number.to_string()
    } else {
        format!("K{number}")
    }
}

/// Return the letter for a side, B or W
fn side_letter(side: Side) -> char {
    match side {
        Side::Black => 'B',
        Side::White => 'W',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Board;

    #[test]
    fn start_position_has_twelve_men_each() {
        let board = Board::start_position();

        // Black on squares 1-12, White on 21-32, no kings, Black to move
        assert_eq!(board.men(Side::Black), 0x0000_0FFF);
        assert_eq!(board.men(Side::White), 0xFFF0_0000);
        assert!(is_empty(board.kings(Side::Black) | board.kings(Side::White)));
        assert_eq!(board.side_to_move(), Side::Black);
    }

    #[test]
    fn positions_round_trip() {
        let positions = [
            START_FEN,
            "W:W18,K30:B9,K14",
            "B:WK1,K32:BK5",
            // A side with no pieces left
            "W:W:B12",
        ];

        for fen in positions {
            let board = Board::from_fen(fen).unwrap();
            assert_eq!(board.to_fen(), fen);
        }
    }

    #[test]
    fn kings_go_on_the_king_bitboards() {
        let board = Board::from_fen("B:W21,K30:B1,K5").unwrap();

        // Notation squares are one higher than their bit index
        assert_eq!(board.men(Side::White), square_mask(20));
        assert_eq!(board.kings(Side::White), square_mask(29));
        assert_eq!(board.men(Side::Black), square_mask(0));
        assert_eq!(board.kings(Side::Black), square_mask(4));
    }

    #[test]
    fn extra_spaces_are_ignored() {
        let board = Board::from_fen(" B : W21, 22 : B1, K5 ").unwrap();
        assert_eq!(board.to_fen(), "B:W21,22:B1,K5");
    }

    #[test]
    fn bad_positions_are_rejected() {
        let bad = [
            // Wrong number of fields
            "B:W21",
            // Not a side
            "X:W21:B1",
            // Squares off the board or not numbers
            "B:W0:B1",
            "B:W33:B1",
            "B:Wabc:B1",
            // The same square twice
            "B:W21,21:B1",
            // Both sides on one square
            "B:W21:B21",
            // Lists in the wrong order
            "B:B1:W21",
            // A man sitting on its own promotion row should be a king
            "B:W21:B30",
            "B:W2:B12",
        ];

        for fen in bad {
            assert!(Board::from_fen(fen).is_err(), "'{fen}' should be rejected");
        }
    }
}