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
}
