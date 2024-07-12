use crate::core::voxel::{coord::Coord, direction::Cardinal};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkCoord {
    x: i32,
    z: i32,
}

impl ChunkCoord {
    #[inline]
    pub const fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z
        }
    }

    #[inline]
    pub const fn xz(self) -> (i32, i32) {
        (self.x, self.z)
    }

    #[inline]
    pub const fn block_coord(self) -> Coord {
        Coord::new(self.x * 16, 0, self.z * 16)
    }

    #[inline]
    pub const fn neighbor(self, direction: Cardinal) -> ChunkCoord {
        match direction {
            Cardinal::West => Self::new(self.x - 1, self.z),
            Cardinal::North => Self::new(self.x, self.z - 1),
            Cardinal::East => Self::new(self.x + 1, self.z),
            Cardinal::South => Self::new(self.x, self.z + 1),
        }
    }
}

impl std::ops::Add<ChunkCoord> for ChunkCoord {
    type Output = Self;
    #[inline]
    fn add(self, rhs: ChunkCoord) -> Self::Output {
        Self::new(self.x + rhs.x, self.z + rhs.z)
    }
}

impl std::ops::Add<Cardinal> for ChunkCoord {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Cardinal) -> Self::Output {
        Self::neighbor(self, rhs)
    }
}

impl std::ops::Sub<ChunkCoord> for ChunkCoord {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: ChunkCoord) -> Self::Output {
        Self::new(self.x - rhs.x, self.z - rhs.z)
    }
}

impl std::ops::Mul<i32> for ChunkCoord {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.z * rhs)
    }
}

impl std::ops::Div<i32> for ChunkCoord {
    type Output = Self;
    #[inline]
    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.x / rhs, self.z / rhs)
    }
}

impl std::ops::Rem<i32> for ChunkCoord {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: i32) -> Self::Output {
        Self::new(self.x % rhs, self.z % rhs)
    }
}

impl std::ops::AddAssign<ChunkCoord> for ChunkCoord {
    #[inline]
    fn add_assign(&mut self, rhs: ChunkCoord) {
        self.x += rhs.x;
        self.z += rhs.z;
    }
}

impl std::ops::SubAssign<ChunkCoord> for ChunkCoord {
    #[inline]
    fn sub_assign(&mut self, rhs: ChunkCoord) {
        self.x -= rhs.x;
        self.z -= rhs.z;
    }
}

impl From<ChunkCoord> for (i32, i32) {
    #[inline]
    fn from(value: ChunkCoord) -> Self {
        value.xz()
    }
}

impl From<(i32, i32)> for ChunkCoord {
    #[inline]
    fn from(value: (i32, i32)) -> Self {
        ChunkCoord::new(value.0, value.1)
    }
}