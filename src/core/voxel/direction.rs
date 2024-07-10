use super::coord::Coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    NegX = 0,
    NegY = 1,
    NegZ = 2,
    PosX = 3,
    PosY = 4,
    PosZ = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Cardinal {
    /// -X
    West  = 0,
    /// -Z
    North = 1,
    /// +X
    East  = 2,
    /// +Z
    South = 3,
}

impl Cardinal {
    /// Ordered: West, East, North, South
    /// West and East, North and South are grouped together for certain desirable effects.
    pub const ALL: [Cardinal; 4] = [
        Cardinal::West,
        Cardinal::East,
        Cardinal::North,
        Cardinal::South,
    ];

    pub const fn rotate(self, rotation: i32) -> Self {
        const CARDS: [Cardinal; 4] = [
            Cardinal::West,
            Cardinal::North,
            Cardinal::East,
            Cardinal::South
        ];
        let index = match self {
            Cardinal::West => 0,
            Cardinal::North => 1,
            Cardinal::East => 2,
            Cardinal::South => 3,
        };
        let rot_index = (index + rotation).rem_euclid(4);
        CARDS[rot_index as usize]
    }

    pub const fn invert(self) -> Self {
        match self {
            Cardinal::West => Cardinal::East,
            Cardinal::East => Cardinal::West,
            Cardinal::North => Cardinal::South,
            Cardinal::South => Cardinal::North,
        }
    }

    pub const fn bit(self) -> u8 {
        1 << self as u8
    }

    pub fn iter() -> impl Iterator<Item = Cardinal> {
        Self::ALL.into_iter()
    }
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

    pub const fn invert(self) -> Self {
        match self {
            Direction::NegX => Direction::PosX,
            Direction::NegY => Direction::PosY,
            Direction::NegZ => Direction::PosZ,
            Direction::PosX => Direction::NegX,
            Direction::PosY => Direction::NegY,
            Direction::PosZ => Direction::NegZ,
        }
    }

    pub const fn bit(self) -> u8 {
        1 << self as u8
    }

    pub fn iter() -> impl Iterator<Item = Direction> {
        Self::ALL.into_iter()
    }
}