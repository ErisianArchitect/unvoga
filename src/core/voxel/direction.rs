use bytemuck::NoUninit;

use super::coord::Coord;
// 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NoUninit)]
#[repr(u8)]
pub enum Direction {
    NegX = 4,// 16
    NegY = 3,// 8
    NegZ = 5,// 32
    PosX = 1,// 2
    PosY = 0,// 1
    PosZ = 2,// 4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NoUninit)]
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
    pub const FORWARD: Cardinal = Cardinal::North;
    pub const BACKWARD: Cardinal = Cardinal::South;
    pub const RIGHT: Cardinal = Cardinal::East;
    pub const LEFT: Cardinal = Cardinal::West;
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
    pub const LEFT: Direction = Direction::NegX;
    pub const DOWN: Direction = Direction::NegY;
    pub const FORWARD: Direction = Direction::NegZ;
    pub const RIGHT: Direction = Direction::PosX;
    pub const UP: Direction = Direction::PosY;
    pub const BACKWARD: Direction = Direction::PosZ;

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

impl std::ops::Neg for Cardinal {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        self.invert()
    }
}

impl std::ops::Neg for Direction {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        self.invert()
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::NegX => write!(f, "NegX"),
            Direction::NegY => write!(f, "NegY"),
            Direction::NegZ => write!(f, "NegZ"),
            Direction::PosX => write!(f, "PosX"),
            Direction::PosY => write!(f, "PosY"),
            Direction::PosZ => write!(f, "PosZ"),
        }
    }
}

#[test]
fn inv_test() {
    let dir = -Cardinal::East;
    println!("{dir:?}");
}