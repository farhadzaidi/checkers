use super::side::Side;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Direction {
    /// Return the direction pointing the other way
    pub fn opposite(self) -> Direction {
        match self {
            Direction::UpLeft => Direction::DownRight,
            Direction::UpRight => Direction::DownLeft,
            Direction::DownLeft => Direction::UpRight,
            Direction::DownRight => Direction::UpLeft,
        }
    }
}

pub type DirectionPair = [Direction; 2];
pub type AllDirections = [Direction; 4];

// Perspective: Black pieces at the top and white pieces at the bottom
const BLACK_MAN_FORWARD: DirectionPair = [Direction::DownRight, Direction::DownLeft];
const WHITE_MAN_FORWARD: DirectionPair = [Direction::UpRight, Direction::UpLeft];
pub const KING_DIRECTIONS: AllDirections = [
    Direction::UpRight,
    Direction::UpLeft,
    Direction::DownRight,
    Direction::DownLeft,
];

/// Return the two directions the given side's men move in
pub fn forward(side: Side) -> DirectionPair {
    match side {
        Side::Black => BLACK_MAN_FORWARD,
        Side::White => WHITE_MAN_FORWARD,
    }
}
