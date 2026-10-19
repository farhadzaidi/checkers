use super::bitboard::{Bitboard, EMPTY_BITBOARD, Square, is_empty, pop_lsb, square_mask};
use super::moves::Move;
use super::side::Side;

// Reading and writing PDN, the standard text format for checkers: square numbers, positions (FEN)
// and moves. Everything here works on text and plain data; Board and Move turn it into the real thing.
//
// PDN numbers the dark squares 1-32, starting from Black's side of the board. We store them as
// bit indexes 0-31, so square 22 in notation is square 21 in the code.

/// Parse a square number from 1 to 32 into its bit index
pub fn parse_square(text: &str) -> Result<Square, String> {
    match text.trim().parse::<Square>() {
        Ok(number @ 1..=32) => Ok(number - 1),
        _ => Err(format!("expected a square from 1 to 32, found '{text}'")),
    }
}

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
pub const START_FEN: &str = "B:W21,22,23,24,25,26,27,28,29,30,31,32:B1,2,3,4,5,6,7,8,9,10,11,12";

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
        return Err(format!(
            "expected 3 fields split by ':', found {}",
            fields.len()
        ));
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

// Moves are written with the same square numbers:
//     9-13       a simple step
//     15x24      a capture, from its start to its final square
//     6x13x22    a capture with every square it lands on, for when two captures share their ends

/// A move as written, before it's matched to a legal move in a position
pub struct ParsedMove {
    pub from: Square,
    // Every square written after the start: just the final square, or each square a capture lands on
    pub landings: Vec<Square>,
    pub is_capture: bool,
}

/// Write a move in standard notation, like 11-15 or 15x24
pub fn format_move(mv: Move) -> String {
    // Notation numbers squares 1-32, but we store them as bits 0-31
    let from = mv.from() + 1;
    let to = mv.to() + 1;

    // Captures use an x, simple steps use a dash
    let separator = if mv.is_capture() { 'x' } else { '-' };
    format!("{from}{separator}{to}")
}

/// Parse move notation into its squares and whether it's a capture
///     "9-13" -> from 8, landing on [12], a step
///     "6x13x22" -> from 5, landing on [12, 21], a capture
pub fn parse_move(notation: &str) -> Result<ParsedMove, String> {
    // Captures use an x, simple steps use a dash
    let notation = notation.trim();
    let is_capture = notation.contains('x');
    let separator = if is_capture { 'x' } else { '-' };
    let parts: Vec<&str> = notation.split(separator).collect();

    // A step goes between two squares; a capture can list every square it lands on
    let too_few = parts.len() < 2;
    let step_with_extra_squares = !is_capture && parts.len() > 2;
    if too_few || step_with_extra_squares {
        return Err(format!(
            "expected a move like 9-13, 15x24 or 6x13x22, found '{notation}'"
        ));
    }

    let from = parse_square(parts[0])?;
    let mut landings = Vec::new();
    for part in &parts[1..] {
        landings.push(parse_square(part)?);
    }

    Ok(ParsedMove {
        from,
        landings,
        is_capture,
    })
}

/// Return true if the move starts and lands where the parsed notation says. The path is the
/// squares the move lands on, which matter when the notation writes every one of them
///     from 5, landing on [21] -> any capture from 5 that ends on 21
///     from 5, landing on [12, 21] -> only the capture that lands on 12, then 21
pub fn describes(mv: Move, parsed: &ParsedMove, path: &[Square]) -> bool {
    if mv.from() != parsed.from || mv.is_capture() != parsed.is_capture {
        return false;
    }

    // Just the start and end written: only the final square has to match
    if let [to] = parsed.landings.as_slice() {
        return mv.to() == *to;
    }

    // Every landing written: the whole route has to match
    path == parsed.landings
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
        assert!(is_empty(
            board.kings(Side::Black) | board.kings(Side::White)
        ));
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

    #[test]
    fn steps_have_a_start_and_one_landing() {
        let parsed = parse_move("9-13").unwrap();
        assert_eq!(parsed.from, 8);
        assert_eq!(parsed.landings, [12]);
        assert!(!parsed.is_capture);
    }

    #[test]
    fn captures_can_list_every_landing() {
        let parsed = parse_move("6x13x22").unwrap();
        assert_eq!(parsed.from, 5);
        assert_eq!(parsed.landings, [12, 21]);
        assert!(parsed.is_capture);
    }

    #[test]
    fn bad_moves_are_rejected() {
        // One square, a step with extra squares, squares off the board, and not numbers
        for notation in ["9", "9-13-17", "0-4", "9-33", "9x", "nine-13"] {
            assert!(parse_move(notation).is_err(), "{notation} was accepted");
        }
    }
}
