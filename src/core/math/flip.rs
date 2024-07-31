#![allow(unused)]
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

    pub const fn new(x: bool, y: bool, z: bool) -> Self {
        Self((x as u8) | ((y as u8) << 1) | ((z as u8) << 2))
    }
    
    pub const fn x(self) -> bool {
        self.0 & Flip::X.0 == Flip::X.0
    }

    
    pub const fn y(self) -> bool {
        self.0 & Flip::Y.0 == Flip::Y.0
    }

    
    pub const fn z(self) -> bool {
        self.0 & Flip::Z.0 == Flip::Z.0
    }

    pub const fn flip(self, flip: Flip) -> Self {
        Self::new(self.x() ^ flip.x(), self.y() ^ flip.y(), self.z() ^ flip.z())
    }

    pub fn set_x(&mut self, value: bool) -> bool {
        let old = self.x();
        if value {
            self.0 = self.0 | Self::X.0;
        } else {
            self.0 = self.0 & Self::YZ.0;
        }
        old
    }

    pub fn set_y(&mut self, value: bool) -> bool {
        let old = self.y();
        if value {
            self.0 = self.0 | Self::Y.0;
        } else {
            self.0 = self.0 & Self::XZ.0;
        }
        old
    }

    pub fn set_z(&mut self, value: bool) -> bool {
        let old = self.z();
        if value {
            self.0 = self.0 | Self::Z.0;
        } else {
            self.0 = self.0 & Self::XY.0;
        }
        old
    }

    pub const fn flip_x(mut self) -> Self {
        self.0 = self.0 ^ Flip::X.0;
        self
    }

    pub const fn flip_y(mut self) -> Self {
        self.0 = self.0 ^ Flip::Y.0;
        self
    }

    pub const fn flip_z(mut self) -> Self {
        self.0 = self.0 ^ Flip::Z.0;
        self
    }

    pub fn invert_x(&mut self) -> bool {
        let old = self.x();
        self.set_x(!old)
    }

    pub fn invert_y(&mut self) -> bool {
        let old = self.y();
        self.set_y(!old)
    }

    pub fn invert_z(&mut self) -> bool {
        let old = self.z();
        self.set_z(!old)
    }

    /// Xors all the bits.
    pub const fn xor(self) -> bool {
        self.x() ^ self.y() ^ self.z()
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
    pub const fn is_flipped(self, face: Direction) -> bool {
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