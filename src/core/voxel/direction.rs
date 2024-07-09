use super::coord::Coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    NegX = 1,
    NegY = 2,
    NegZ = 4,
    PosX = 8,
    PosY = 16,
    PosZ = 32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Cardinal {
    East,	// +X
    West,	// -X
    South,	// +Z
    North,	// -Z
}

impl Direction {
    pub const ALL: [Direction; 6] = [
        Direction::NegX,
        Direction::NegY,
        Direction::NegZ,
        Direction::PosX,
        Direction::PosY,
        Direction::PosZ
    ];

    pub fn bit(self) -> u8 {
        self as u8
    }

    pub fn iter() -> impl Iterator<Item = Direction> {
        Self::ALL.into_iter()
    }
}

impl From<Direction> for Coord {
    fn from(value: Direction) -> Self {
        match value {
            Direction::NegX => Coord::new(-1, 0, 0),
            Direction::NegY => Coord::new(0, -1, 0),
            Direction::NegZ => Coord::new(0, 0, -1),
            Direction::PosX => Coord::new(1, 0, 0),
            Direction::PosY => Coord::new(0, 1, 0),
            Direction::PosZ => Coord::new(0, 0, 1),
        }
    }
}