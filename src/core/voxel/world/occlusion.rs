#![allow(unused)]
use crate::core::voxel::direction::Direction;

macro_rules! make_face_constants {
    ($($name:ident = $dir:ident;)*) => {
        $(
            pub const $name: Self = Occlusion(1 << Direction::$dir as u8);
        )*
    };
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Occlusion(pub u8);

impl Occlusion {
    pub const UNOCCLUDED: Self = Occlusion(0);
    pub const OCCLUDED: Self = Occlusion(0b111111);
    make_face_constants!(
        NEG_X = NegX;
        NEG_Y = NegY;
        NEG_Z = NegZ;
        POS_X = PosX;
        POS_Y = PosY;
        POS_Z = PosZ;
    );
    const FLAGS_MASK: u8 = 0b111111;

    
    pub fn fully_occluded(self) -> bool {
        self == Self::OCCLUDED
    }

    
    pub fn show(&mut self, face: Direction) -> bool {
        let bit = face.bit();
        let old = self.0 & bit == bit;
        self.0 = self.0 & !bit;
        old
    }

    
    pub fn hide(&mut self, face: Direction) -> bool {
        let bit = face.bit();
        let old = self.0 & bit == bit;
        self.0 = self.0 | bit;
        old
    }

    
    pub fn visible(self, face: Direction) -> bool {
        let bit = face.bit();
        self.0 & bit != bit
    }

    
    pub fn hidden(self, face: Direction) -> bool {
        let bit = face.bit();
        self.0 & bit == bit
    }

    /// Returns true if visible.
    pub fn neg_x(self) -> bool {
        self.visible(Direction::NegX)
    }

    /// Returns true if visible.
    pub fn neg_y(self) -> bool {
        self.visible(Direction::NegY)
    }

    /// Returns true if visible.
    pub fn neg_z(self) -> bool {
        self.visible(Direction::NegZ)
    }

    /// Returns true if visible.
    pub fn pos_x(self) -> bool {
        self.visible(Direction::PosX)
    }

    /// Returns true if visible.
    pub fn pos_y(self) -> bool {
        self.visible(Direction::PosY)
    }

    /// Returns true if visible.
    pub fn pos_z(self) -> bool {
        self.visible(Direction::PosZ)
    }
}

impl std::ops::BitOr<Occlusion> for Occlusion {
    type Output = Occlusion;
    
    fn bitor(self, rhs: Occlusion) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd<Occlusion> for Occlusion {
    type Output = Occlusion;
    
    fn bitand(self, rhs: Occlusion) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Sub<Occlusion> for Occlusion {
    type Output = Occlusion;
    
    fn sub(self, rhs: Occlusion) -> Self::Output {
        Self(self.0 & !rhs.0)
    }
}

impl std::ops::BitAnd<Direction> for Occlusion {
    type Output = bool;
    fn bitand(self, rhs: Direction) -> Self::Output {
        self.visible(rhs)
    }
}

impl std::fmt::Display for Occlusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Occlusion(")?;
        Direction::iter().try_fold(false, |mut sep, dir| {
            if self.hidden(dir) {
                if sep {
                    write!(f, "|")?;
                }
                sep = true;
                write!(f, "{dir:?}")?;
            }
            Ok(sep)
        })?;
        write!(f, ")")
    }
}