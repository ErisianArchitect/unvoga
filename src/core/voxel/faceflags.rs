use crate::prelude::Direction;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceFlags(u8);

macro_rules! unit_consts {
    ($(const $name:ident -> $dir:ident;)*) => {
        impl FaceFlags {
            $(
                pub const $name: FaceFlags = FaceFlags(1 << Direction::$dir as u8);
            )*
        }
    };
}
use paste::paste;
macro_rules! unit_getters {
    ($(pub const fn $name:ident => FaceFlags::$cnst:ident;)*) => {
        impl FaceFlags {
            $(
                pub const fn $name(self) -> bool {
                    (self.0 & Self::$cnst.0) != 0
                }
                paste!{
                    pub fn [<set_ $name>](&mut self, value: bool) -> bool {
                        let old = (self.0 & Self::$cnst.0) != 0;
                        if value {
                            self.0 = self.0 | Self::$cnst.0;
                        } else {
                            self.0 = self.0 & !Self::$cnst.0;
                        }
                        old
                    }
                }
            )*
        }
    };
}

unit_consts!(
    const POS_X -> PosX;
    const POS_Y -> PosY;
    const POS_Z -> PosZ;
    const NEG_X -> NegX;
    const NEG_Y -> NegY;
    const NEG_Z -> NegZ;
);

unit_getters!(
    pub const fn pos_x => FaceFlags::POS_X;
    pub const fn pos_y => FaceFlags::POS_Y;
    pub const fn pos_z => FaceFlags::POS_Z;
    pub const fn neg_x => FaceFlags::NEG_X;
    pub const fn neg_y => FaceFlags::NEG_Y;
    pub const fn neg_z => FaceFlags::NEG_Z;
);

impl FaceFlags {
    pub const NONE: FaceFlags = FaceFlags(0);
    pub const ALL: FaceFlags = FaceFlags(0b111111);
    pub const POS: FaceFlags = FaceFlags(FaceFlags::POS_X.0 | FaceFlags::POS_Y.0 | FaceFlags::POS_Z.0);
    pub const NEG: FaceFlags = FaceFlags(FaceFlags::NEG_X.0 | FaceFlags::NEG_Y.0 | FaceFlags::NEG_Z.0);
    pub const fn new(
        pos_x: bool,
        pos_y: bool,
        pos_z: bool,
        neg_x: bool,
        neg_y: bool,
        neg_z: bool
    ) -> Self {
        use Direction::*;
        Self(
            ((pos_x as u8) << PosX as u8) |
            ((pos_y as u8) << PosY as u8) |
            ((pos_z as u8) << PosZ as u8) |
            ((neg_x as u8) << NegX as u8) |
            ((neg_y as u8) << NegY as u8) |
            ((neg_z as u8) << NegZ as u8)
        )
    }

    pub const fn get(self, face: Direction) -> bool {
        (self.0 & (1 << face as u32)) != 0
    }

    pub fn set(&mut self, face: Direction, value: bool) -> bool {
        let old = (self.0 & (1 << face as u32)) != 0;
        if value {
            self.0 = self.0 | (1 << face as u32);
        } else {
            self.0 = self.0 & !(1 << face as u32);
        }
        old
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub const fn any(self) -> bool {
        self.0 != 0
    }

    /// Returns true if either pos_x or neg_x is set.
    pub const fn x_or(self) -> bool {
        (self.0 & Self::POS_X.0) != 0 ||
        (self.0 & Self::NEG_X.0) != 0
    }

    /// Returns true if pos_x and neg_x is set.
    pub const fn x_and(self) -> bool {
        (self.0 & Self::POS_X.0) != 0 &&
        (self.0 & Self::NEG_X.0) != 0
    }

    /// Returns true if pos_x xor neg_x is set.
    pub const fn x_xor(self) -> bool {
        ((self.0 & Self::POS_X.0) != 0) ^
        ((self.0 & Self::NEG_X.0) != 0)
    }

    /// Returns true if either pos_y or neg_y is set.
    pub const fn y_or(self) -> bool {
        (self.0 & Self::POS_Y.0) != 0 ||
        (self.0 & Self::NEG_Y.0) != 0
    }

    /// Returns true if pos_y and neg_y is set.
    pub const fn y_and(self) -> bool {
        (self.0 & Self::POS_Y.0) != 0 &&
        (self.0 & Self::NEG_Y.0) != 0
    }

    /// Returns true if pos_y xor neg_y is set.
    pub const fn y_xor(self) -> bool {
        ((self.0 & Self::POS_Y.0) != 0) ^
        ((self.0 & Self::NEG_Y.0) != 0)
    }

    /// Returns true if either pos_z or neg_z is set.
    pub const fn z_or(self) -> bool {
        (self.0 & Self::POS_Z.0) != 0 ||
        (self.0 & Self::NEG_Z.0) != 0
    }

    /// Returns true if pos_z and neg_z is set.
    pub const fn z_and(self) -> bool {
        (self.0 & Self::POS_Z.0) != 0 &&
        (self.0 & Self::NEG_Z.0) != 0
    }

    /// Returns true if pos_z xor neg_z is set.
    pub const fn z_xor(self) -> bool {
        ((self.0 & Self::POS_Z.0) != 0) ^
        ((self.0 & Self::NEG_Z.0) != 0)
    }

    /// Returns true if all faces are set to true.
    pub const fn all(self) -> bool {
        self.0 == 0b111111
    }

    pub const fn none(self) -> bool {
        self.0 == 0
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0b111111)
    }
}

impl std::ops::BitOr<FaceFlags> for FaceFlags {
    type Output = Self;
    fn bitor(self, rhs: FaceFlags) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd<FaceFlags> for FaceFlags {
    type Output = Self;
    fn bitand(self, rhs: FaceFlags) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Not for FaceFlags {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0 & 0b111111)
    }
}

impl std::fmt::Display for FaceFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FaceFlags(")?;
        let mut sep = false;
        if self.pos_x() {
            write!(f, "PosX")?;
            sep = true;
        }
        if self.pos_y() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "PosY")?;
            sep = true;
        }
        if self.pos_z() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "PosZ")?;
            sep = true;
        }
        if self.neg_x() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "NegX")?;
            sep = true;
        }
        if self.neg_y() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "NegY")?;
            sep = true;
        }
        if self.neg_z() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "NegZ")?;
            sep = true;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fmt_test() {
        let mut flags = FaceFlags::POS_X | FaceFlags::NEG_X;
        println!("{flags}");
    }
}