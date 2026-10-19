use super::side::Side;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Piece {
    BlackMan,
    BlackKing,
    WhiteMan,
    WhiteKing,
}

impl Piece {
    /// Return the man for the given side
    pub fn man(side: Side) -> Piece {
        match side {
            Side::Black => Piece::BlackMan,
            Side::White => Piece::WhiteMan,
        }
    }

    /// Return the king for the given side
    pub fn king(side: Side) -> Piece {
        match side {
            Side::Black => Piece::BlackKing,
            Side::White => Piece::WhiteKing,
        }
    }

    /// Return the side this piece belongs to
    pub fn side(self) -> Side {
        match self {
            Piece::BlackMan | Piece::BlackKing => Side::Black,
            Piece::WhiteMan | Piece::WhiteKing => Side::White,
        }
    }
}
