use bytemuck::NoUninit;

use crate::prelude::*;

use super::coord::Coord;
// 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NoUninit)]
#[repr(u8)]
pub enum Direction {
    // The order of the ids is a little strange because I wanted PosY to be the first
    // index so that I could have Rotation(0) be the default rotation and have it point to PosY.
    // I could have done some remapping for the Rotation, but then it wouldn't have had a 1:1 representation.
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
    pub const INDEX_ORDER: [Direction; 6] = [
        Direction::PosY,
        Direction::PosX,
        Direction::PosZ,
        Direction::NegY,
        Direction::NegX,
        Direction::NegZ,
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

    pub fn flip(self, flip: Flip) -> Self {
        use Direction::*;
        match self {
            NegX if flip.x() => PosX,
            NegY if flip.y() => PosY,
            NegZ if flip.z() => PosZ,
            PosX if flip.x() => NegX,
            PosY if flip.y() => NegY,
            PosZ if flip.z() => NegZ,
            _ => self
        }
    }

    /// You can also use [Rotation::reface] to achieve the same effect (which is actually what this method does).
    #[inline(always)]
    pub fn rotate(self, rotation: Rotation) -> Self {
        rotation.reface(self)
    }

    pub const fn bit(self) -> u8 {
        1 << self as u8
    }

    pub fn iter() -> impl Iterator<Item = Direction> {
        Self::ALL.into_iter()
    }

    pub fn iter_index_order() -> impl Iterator<Item = Direction> {
        Self::INDEX_ORDER.into_iter()
    }

    /// On a non-oriented cube, each face has an "up" face. That's the face
    /// whose normal points to the top of the given face's UV plane.
    pub fn up(self) -> Direction {
        use Direction::*;
        match self {
            NegX => PosY,
            NegY => PosZ,
            NegZ => PosY,
            PosX => PosY,
            PosY => NegZ,
            PosZ => PosY,
        }
    }

    /// On a non-oriented cube, each face has a "down" face. That's the face
    /// whose normal points to the bottom of the given face's UV plane.
    pub fn down(self) -> Direction {
        use Direction::*;
        match self {
            NegX => NegY,
            NegY => NegZ,
            NegZ => NegY,
            PosX => NegY,
            PosY => PosZ,
            PosZ => NegY,
        }
    }

    /// On a non-oriented cube, each face has a "left" face. That's the face
    /// whose normal points to the left of the given face's UV plane.
    pub fn left(self) -> Direction {
        use Direction::*;
        match self {
            NegX => NegZ,
            NegY => NegX,
            NegZ => PosX,
            PosX => PosZ,
            PosY => NegX,
            PosZ => NegX,
        }
    }

    /// On a non-oriented cube, each face has a "right" face. That's the face
    /// whose normal points to the right of the given face's UV plane.
    pub fn right(self) -> Direction {
        use Direction::*;
        match self {
            NegX => PosZ,
            NegY => PosX,
            NegZ => NegX,
            PosX => NegZ,
            PosY => PosX,
            PosZ => PosX,
        }
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