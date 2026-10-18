use super::bitboard::Bitboard;
use super::side::Side;

/// Black men crown on the bottom row, White men on the top row
pub const BLACK_PROMOTION_ROW: Bitboard = 0xF000_0000;
pub const WHITE_PROMOTION_ROW: Bitboard = 0x0000_000F;

/// Return the row where the given side's men get crowned
pub fn promotion_row(side: Side) -> Bitboard {
    match side {
        Side::Black => BLACK_PROMOTION_ROW,
        Side::White => WHITE_PROMOTION_ROW,
    }
}
