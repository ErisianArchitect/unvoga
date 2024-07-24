use bevy::math::Vec3;

use crate::prelude::Direction;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flip(pub u8);

impl Flip {
    pub const X: Flip = Flip(0b001);
    pub const XY: Flip = Flip(0b011);
    pub const XZ: Flip = Flip(0b101);
    pub const Y: Flip = Flip(0b010);
    pub const YZ: Flip = Flip(0b110);
    pub const Z: Flip = Flip(0b100);
    pub const XYZ: Flip = Flip(0b111);
    pub const ALL: Flip = Flip::XYZ;
    pub const NONE: Flip = Flip(0b000);

    
    pub fn x(self) -> bool {
        self & Flip::X == Flip::X
    }

    
    pub fn y(self) -> bool {
        self & Flip::Y == Flip::Y
    }

    
    pub fn z(self) -> bool {
        self & Flip::Z == Flip::Z
    }

    pub fn flip_coord<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T, T)> + From<(T, T, T)>>(self, mut value: C) -> C {
        let (mut x, mut y, mut z): (T, T, T) = value.into();
        if self.x() {
            x = -x;
        }
        if self.y() {
            y = -y;
        }
        if self.z() {
            z = -z;
        }
        C::from((x, y, z))
    }

    /// Determines if a face is on an axis that is flipped.
    pub fn is_flipped(self, face: Direction) -> bool {
        if self.0 == 0 {
            return false;
        }
        use Direction::*;
        match face {
            NegX | PosX if self.x() => true,
            NegY | PosY if self.y() => true,
            NegZ | PosZ if self.z() => true,
            _ => false,
        }
    }
}

impl std::ops::BitOr<Flip> for Flip {
    type Output = Self;
    
    
    fn bitor(self, rhs: Flip) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign<Flip> for Flip {
    
    fn bitor_assign(&mut self, rhs: Flip) {
        *self = *self | rhs;
    }
}

impl std::ops::BitAndAssign<Flip> for Flip {
    
    fn bitand_assign(&mut self, rhs: Flip) {
        *self = *self & rhs;
    }
}

impl std::ops::Add<Flip> for Flip {
    type Output = Flip;
    
    fn add(self, rhs: Flip) -> Self::Output {
        self | rhs
    }
}

impl std::ops::Sub<Flip> for Flip {
    type Output = Flip;
    
    fn sub(self, rhs: Flip) -> Self::Output {
        self & !rhs
    }
}

impl std::ops::BitAnd<Flip> for Flip {
    type Output = Self;
    
    fn bitand(self, rhs: Flip) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Not for Flip {
    type Output = Self;
    
    fn not(self) -> Self::Output {
        Self(!self.0 & 0b111)
    }
}

impl std::fmt::Display for Flip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Flip(")?;
        let mut sep = false;
        if self.x() {
            write!(f, "X")?;
            sep = true;
        }
        if self.y() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "Y")?;
        }
        if self.z() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "Z")?;
        }
        write!(f, ")")
    }
}