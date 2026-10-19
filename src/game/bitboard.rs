use super::direction::Direction;

/// One bit per dark square, bit 0 is square 1 in notation
pub type Bitboard = u32;
pub const EMPTY_BITBOARD: Bitboard = 0;

/// A dark square's bit index, from 0 to 31
pub type Square = u8;

/// Return true if the bitboard has no squares set
pub fn is_empty(bb: Bitboard) -> bool {
    bb == EMPTY_BITBOARD
}

/// Return a bitboard with only the given square set
pub fn square_mask(square: Square) -> Bitboard {
    debug_assert!(square < 32, "square {square} is off the board");
    1 << square
}

/// Return the square of the lowest set bit, which is the square itself for a single-square mask
pub fn to_square(bb: Bitboard) -> Square {
    // An empty bitboard has no lowest bit, and would give the invalid square 32
    debug_assert!(!is_empty(bb), "to_square on an empty bitboard");
    bb.trailing_zeros() as Square
}

/// Return the row and column of a square on the full 8x8 board, counted from the top left
pub fn row_and_col(square: Square) -> (usize, usize) {
    debug_assert!(square < 32, "square {square} is off the board");

    // Each row holds 4 dark squares: odd columns on even (right-aligned) rows,
    // even columns on odd (left-aligned) rows
    //     square 0 -> row 0, column 1
    //     square 4 -> row 1, column 0
    let row = square as usize / 4;
    let col = (square as usize % 4) * 2 + (1 - row % 2);
    (row, col)
}

/// Return the square at a row and column of the full 8x8 board, or None for a light square
pub fn square_at(row: usize, col: usize) -> Option<Square> {
    debug_assert!(
        row < 8 && col < 8,
        "row {row}, column {col} is off the board"
    );

    // Pieces only stand on the dark squares, where row + col is odd
    if (row + col).is_multiple_of(2) {
        return None;
    }
    Some((row * 4 + col / 2) as Square)
}

// Each row has 4 dark squares (one hex digit), counted from the top row.
// Rows alternate between two shapes:
//   - Right-aligned: the dark squares sit to the right, so the last one is on
//     the board's right edge and the left corner is a light square
//   - Left-aligned: the dark squares sit to the left, so the first one is on
//     the board's left edge and the right corner is a light square
//
// The row type a piece starts on decides how far to shift for a diagonal step.
// A piece can only step off the side its row touches, so rightward steps drop
// the right edge squares and leftward steps drop the left edge squares.

const RIGHT_ALIGNED_ROWS: Bitboard = 0x0F0F_0F0F;
const RIGHT_EDGE: Bitboard = 0x0808_0808;
const RIGHT_ALIGNED_ROWS_NOT_RIGHT_EDGE: Bitboard = RIGHT_ALIGNED_ROWS & !RIGHT_EDGE;

const LEFT_ALIGNED_ROWS: Bitboard = 0xF0F0_F0F0;
const LEFT_EDGE: Bitboard = 0x1010_1010;
const LEFT_ALIGNED_ROWS_NOT_LEFT_EDGE: Bitboard = LEFT_ALIGNED_ROWS & !LEFT_EDGE;

/// Move every piece one step up and to the right
fn up_right(bb: Bitboard) -> Bitboard {
    ((bb & RIGHT_ALIGNED_ROWS_NOT_RIGHT_EDGE) >> 3) | ((bb & LEFT_ALIGNED_ROWS) >> 4)
}

/// Move every piece one step up and to the left
fn up_left(bb: Bitboard) -> Bitboard {
    ((bb & RIGHT_ALIGNED_ROWS) >> 4) | ((bb & LEFT_ALIGNED_ROWS_NOT_LEFT_EDGE) >> 5)
}

/// Move every piece one step down and to the right
fn down_right(bb: Bitboard) -> Bitboard {
    ((bb & RIGHT_ALIGNED_ROWS_NOT_RIGHT_EDGE) << 5) | ((bb & LEFT_ALIGNED_ROWS) << 4)
}

/// Move every piece one step down and to the left
fn down_left(bb: Bitboard) -> Bitboard {
    ((bb & RIGHT_ALIGNED_ROWS) << 4) | ((bb & LEFT_ALIGNED_ROWS_NOT_LEFT_EDGE) << 3)
}

/// Move every piece one diagonal step in the given direction
pub fn shift(bb: Bitboard, direction: Direction) -> Bitboard {
    match direction {
        Direction::UpLeft => up_left(bb),
        Direction::UpRight => up_right(bb),
        Direction::DownLeft => down_left(bb),
        Direction::DownRight => down_right(bb),
    }
}

/// Remove the lowest set bit from a non-empty bitboard and return its square
pub fn pop_lsb(bb: &mut Bitboard) -> Square {
    debug_assert!(!is_empty(*bb), "pop_lsb on an empty bitboard");
    let square = to_square(*bb);
    *bb &= *bb - 1;
    square
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_DIRECTIONS: [(Direction, i32, i32); 4] = [
        // Direction, then how it changes the row and column on the 8x8 board
        (Direction::UpLeft, -1, -1),
        (Direction::UpRight, -1, 1),
        (Direction::DownLeft, 1, -1),
        (Direction::DownRight, 1, 1),
    ];

    /// Return a square's row and column as signed numbers, so a step off the board can go negative
    fn row_col(square: Square) -> (i32, i32) {
        let (row, col) = row_and_col(square);
        (row as i32, col as i32)
    }

    #[test]
    fn every_shift_moves_one_diagonal_step_or_falls_off() {
        for square in 0..32 {
            let (row, col) = row_col(square);

            for (direction, row_step, col_step) in ALL_DIRECTIONS {
                let shifted = shift(square_mask(square), direction);
                let target_row = row + row_step;
                let target_col = col + col_step;
                let on_board = (0..8).contains(&target_row) && (0..8).contains(&target_col);

                if on_board {
                    // Exactly one piece, on the diagonal neighbour
                    assert_eq!(shifted.count_ones(), 1, "square {square} {direction:?}");
                    assert_eq!(
                        row_col(to_square(shifted)),
                        (target_row, target_col),
                        "square {square} {direction:?}"
                    );
                } else {
                    // Off the edge, so the piece disappears instead of wrapping
                    assert!(is_empty(shifted), "square {square} {direction:?} wrapped");
                }
            }
        }
    }

    #[test]
    fn shifting_many_pieces_matches_shifting_each_one() {
        for (direction, _, _) in ALL_DIRECTIONS {
            // Shift all 32 squares at once, then compare to shifting them one by one
            let all_at_once = shift(!EMPTY_BITBOARD, direction);
            let one_by_one = (0..32)
                .map(|square| shift(square_mask(square), direction))
                .fold(EMPTY_BITBOARD, |result, shifted| result | shifted);

            assert_eq!(all_at_once, one_by_one, "{direction:?}");
        }
    }

    #[test]
    fn pop_lsb_returns_squares_lowest_first() {
        let mut bb = square_mask(3) | square_mask(17) | square_mask(30);

        assert_eq!(pop_lsb(&mut bb), 3);
        assert_eq!(pop_lsb(&mut bb), 17);
        assert_eq!(pop_lsb(&mut bb), 30);
        assert!(is_empty(bb));
    }

    #[test]
    fn square_mask_and_to_square_round_trip() {
        for square in 0..32 {
            assert_eq!(to_square(square_mask(square)), square);
        }
    }

    #[test]
    fn corner_squares_sit_where_the_board_has_them() {
        // Square 1 in notation is in the top row's second column, and 32 in the bottom row's seventh
        assert_eq!(row_and_col(0), (0, 1));
        assert_eq!(row_and_col(3), (0, 7));
        assert_eq!(row_and_col(4), (1, 0));
        assert_eq!(row_and_col(31), (7, 6));
    }

    #[test]
    fn row_and_col_and_square_at_round_trip() {
        for square in 0..32 {
            let (row, col) = row_and_col(square);
            assert_eq!(square_at(row, col), Some(square));
        }
    }

    #[test]
    fn light_squares_hold_no_square() {
        // The top left corner is light, and so is every square where row + col is even
        assert_eq!(square_at(0, 0), None);
        assert_eq!(square_at(7, 7), None);
        assert_eq!(square_at(3, 5), None);
    }
}
