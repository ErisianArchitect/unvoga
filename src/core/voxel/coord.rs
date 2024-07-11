use crate::core::math::grid;

use super::{direction::{Cardinal, Direction}, world::chunkcoord::ChunkCoord};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

impl Coord {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x,
            y,
            z
        }
    }

    #[inline]
    pub const fn splat(splat: i32) -> Self {
        Self::new(splat, splat, splat)
    }

    pub fn neighbors(self) -> impl Iterator<Item = (Direction, Coord)> {
        Direction::iter().filter_map(move |dir| {
            let dir_coord: Coord = dir.into();
            let coord = Coord::new(
                self.x.checked_add(dir_coord.x)?,
                self.y.checked_add(dir_coord.y)?,
                self.z.checked_add(dir_coord.z)?
            );
            Some((dir, coord))
        })
    }

    #[inline]
    pub const fn chunk_coord(self) -> ChunkCoord {
        ChunkCoord::new(self.x / 16, self.z / 16)
    }

    #[inline]
    pub const fn section_coord(self) -> Coord {
        Self::new(self.x / 16, self.y / 16, self.z / 16)
    }

    #[inline]
    pub const fn rem_euclid(self, rhs: i32) -> Self {
        Self::new(
            self.x.rem_euclid(rhs),
            self.y.rem_euclid(rhs),
            self.z.rem_euclid(rhs)
        )
    }

    pub const fn snap(self, snap: Coord, offset: Coord) -> Self {
        Self {
            x: grid::snap(self.x, snap.x, offset.x),
            y: grid::snap(self.y, snap.y, offset.y),
            z: grid::snap(self.z, snap.z, offset.z)
        }
    }

    pub const fn snapi(self, snap: i32, offset: i32) -> Self {
        Self {
            x: grid::snap(self.x, snap, offset),
            y: grid::snap(self.y, snap, offset),
            z: grid::snap(self.z, snap, offset)
        }
    }
}

impl std::ops::Add<Coord> for Coord {
    type Output = Coord;
    #[inline]
    fn add(self, rhs: Coord) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}

impl std::ops::Sub<Coord> for Coord {
    type Output = Coord;
    #[inline]
    fn sub(self, rhs: Coord) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}

impl std::ops::Mul<i32> for Coord {
    type Output = Coord;
    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs
        }
    }
}

impl std::ops::Div<i32> for Coord {
    type Output = Coord;
    #[inline]
    fn div(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs
        }
    }
}

impl std::ops::Rem<i32> for Coord {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x % rhs,
            y: self.y % rhs,
            z: self.z % rhs
        }
    }
}

impl std::ops::Add<Direction> for Coord {
    type Output = Coord;
    #[inline]
    fn add(self, rhs: Direction) -> Self::Output {
        match rhs {
            Direction::NegX => Coord::new(self.x - 1, self.y, self.z),
            Direction::NegY => Coord::new(self.x, self.y - 1, self.z),
            Direction::NegZ => Coord::new(self.x, self.y, self.z - 1),
            Direction::PosX => Coord::new(self.x + 1, self.y, self.z),
            Direction::PosY => Coord::new(self.x, self.y + 1, self.z),
            Direction::PosZ => Coord::new(self.x, self.y, self.z + 1),
        }
    }
}

impl std::ops::Add<Cardinal> for Coord {
    type Output = Coord;
    #[inline]
    fn add(self, rhs: Cardinal) -> Self::Output {
        match rhs {
            Cardinal::West => Coord::new(self.x - 1, self.y, self.z),
            Cardinal::East => Coord::new(self.x + 1, self.y, self.z),
            Cardinal::North => Coord::new(self.x, self.y, self.z - 1),
            Cardinal::South => Coord::new(self.x, self.y, self.z + 1),
        }
    }
}

impl From<Cardinal> for Coord {
    #[inline]
    fn from(value: Cardinal) -> Self {
        match value {
            Cardinal::West  => Coord::new(-1,  0,  0),
            Cardinal::East  => Coord::new( 1,  0,  0),
            Cardinal::North => Coord::new( 0,  0, -1),
            Cardinal::South => Coord::new( 0,  0,  1),
        }
    }
}

impl From<Direction> for Coord {
    #[inline]
    fn from(value: Direction) -> Self {
        match value {
            Direction::NegX => Coord::new(-1,  0,  0),
            Direction::NegY => Coord::new( 0, -1,  0),
            Direction::NegZ => Coord::new( 0,  0, -1),
            Direction::PosX => Coord::new( 1,  0,  0),
            Direction::PosY => Coord::new( 0,  1,  0),
            Direction::PosZ => Coord::new( 0,  0,  1),
        }
    }
}

impl std::fmt::Display for Coord {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<Coord> for (i32, i32, i32) {
    #[inline]
    fn from(value: Coord) -> Self {
        (value.x, value.y, value.z)
    }
}

impl From<(i32, i32, i32)> for Coord {
    #[inline]
    fn from(value: (i32, i32, i32)) -> Self {
        Coord::new(value.0, value.1, value.2)
    }
}

#[test]
fn neighbors_test() {
    let max = Coord::new(3, 1, 4);
    max.neighbors().for_each(|(dir, coord)| {
        println!("{dir:?}");
    });
}