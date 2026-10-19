use super::bitboard::Bitboard;

/// Black men crown on the bottom row, White men on the top row
const BLACK_PROMOTION_ROW: Bitboard = 0xF000_0000;
const WHITE_PROMOTION_ROW: Bitboard = 0x0000_000F;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Black,
    White,
}

impl Side {
    /// Return the other side
    pub fn opposite(self) -> Side {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }

    /// Return the row where this side's men get crowned
    pub fn promotion_row(self) -> Bitboard {
        match self {
            Side::Black => BLACK_PROMOTION_ROW,
            Side::White => WHITE_PROMOTION_ROW,
        }
    }
}
