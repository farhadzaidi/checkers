use super::direction::Direction;
use super::types::{Bitboard, Square};

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

/// Move every piece in `bb` one step up and to the right
fn up_right(bb: Bitboard) -> Bitboard {
    ((bb & RIGHT_ALIGNED_ROWS_NOT_RIGHT_EDGE) >> 3) | ((bb & LEFT_ALIGNED_ROWS) >> 4)
}

/// Move every piece in `bb` one step up and to the left
fn up_left(bb: Bitboard) -> Bitboard {
    ((bb & RIGHT_ALIGNED_ROWS) >> 4) | ((bb & LEFT_ALIGNED_ROWS_NOT_LEFT_EDGE) >> 5)
}

/// Move every piece in `bb` one step down and to the right
fn down_right(bb: Bitboard) -> Bitboard {
    ((bb & RIGHT_ALIGNED_ROWS_NOT_RIGHT_EDGE) << 5) | ((bb & LEFT_ALIGNED_ROWS) << 4)
}

/// Move every piece in `bb` one step down and to the left
fn down_left(bb: Bitboard) -> Bitboard {
    ((bb & RIGHT_ALIGNED_ROWS) << 4) | ((bb & LEFT_ALIGNED_ROWS_NOT_LEFT_EDGE) << 3)
}

/// Move every piece in `bb` one diagonal step in `direction`
pub fn shift(bb: Bitboard, direction: Direction) -> Bitboard {
    match direction {
        Direction::UpLeft => up_left(bb),
        Direction::UpRight => up_right(bb),
        Direction::DownLeft => down_left(bb),
        Direction::DownRight => down_right(bb),
    }
}

/// Remove the lowest set bit from `bb` and return its square. `bb` must not be empty
pub fn pop_lsb(bb: &mut Bitboard) -> Square {
    let square = bb.trailing_zeros() as Square;
    *bb &= *bb - 1;
    square
}
