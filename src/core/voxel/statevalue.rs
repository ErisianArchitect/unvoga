use bevy::math::{IVec2, IVec3};

use crate::{core::util::traits::StrToOwned, prelude::*};

use super::faceflags::FaceFlags;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StateValue {
    #[default]
    Null = 0,
    Int(i64) = 1,
    Bool(bool) = 2,
    String(String) = 3,
    Direction(Direction) = 4,
    Cardinal(Cardinal) = 5,
    Rotation(Rotation) = 6,
    Flip(Flip) = 7,
    Orientation(Orientation) = 8,
    Axis(Axis) = 9,
    Coord2(IVec2) = 10,
    Coord3(IVec3) = 11,
    FaceFlags(FaceFlags) = 12,
    BitFlags8(BitFlags8) = 13,
    BitFlags16(BitFlags16) = 14,
    BitFlags32(BitFlags32) = 15,
    BitFlags64(BitFlags64) = 16,
}

impl StateValue {
    
    pub fn id(&self) -> u8 {
        match self {
            StateValue::Null => 0,
            StateValue::Int(_) => 1,
            StateValue::Bool(_) => 2,
            StateValue::String(_) => 3,
            StateValue::Direction(_) => 4,
            StateValue::Cardinal(_) => 5,
            StateValue::Rotation(_) => 6,
            StateValue::Flip(_) => 7,
            StateValue::Orientation(_) => 8,
            StateValue::Axis(_) => 9,
            StateValue::Coord2(_) => 10,
            StateValue::Coord3(_) => 11,
            StateValue::FaceFlags(_) => 12,
            StateValue::BitFlags8(_) => 13,
            StateValue::BitFlags16(_) => 14,
            StateValue::BitFlags32(_) => 15,
            StateValue::BitFlags64(_) => 16,
        }
    }
}

impl Readable for StateValue {
    fn read_from<R: std::io::Read>(reader: &mut R) -> crate::prelude::VoxelResult<Self> {
        let id = u8::read_from(reader)?;
        Ok(match id {
            0 => StateValue::Null,
            1 => StateValue::Int(i64::read_from(reader)?),
            2 => StateValue::Bool(bool::read_from(reader)?),
            3 => StateValue::String(String::read_from(reader)?),
            4 => StateValue::Direction(Direction::read_from(reader)?),
            5 => StateValue::Cardinal(Cardinal::read_from(reader)?),
            6 => StateValue::Rotation(Rotation::read_from(reader)?),
            7 => StateValue::Flip(Flip::read_from(reader)?),
            8 => StateValue::Orientation(Orientation::read_from(reader)?),
            9 => StateValue::Axis(Axis::read_from(reader)?),
            10 => StateValue::Coord2(IVec2::read_from(reader)?),
            11 => StateValue::Coord3(IVec3::read_from(reader)?),
            12 => StateValue::FaceFlags(FaceFlags::read_from(reader)?),
            13 => StateValue::BitFlags8(BitFlags8::read_from(reader)?),
            14 => StateValue::BitFlags16(BitFlags16::read_from(reader)?),
            15 => StateValue::BitFlags32(BitFlags32::read_from(reader)?),
            16 => StateValue::BitFlags64(BitFlags64::read_from(reader)?),
            _ => return Err(crate::prelude::VoxelError::InvalidBinaryFormat),
        })
    }
}

impl Writeable for StateValue {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> crate::prelude::VoxelResult<u64> {
        self.id().write_to(writer)?;
        Ok(match self {
            StateValue::Null => return Ok(1),
            StateValue::Int(value) => value.write_to(writer)?,
            StateValue::Bool(value) => value.write_to(writer)?,
            StateValue::String(value) => value.write_to(writer)?,
            StateValue::Direction(value) => value.write_to(writer)?,
            StateValue::Cardinal(value) => value.write_to(writer)?,
            StateValue::Rotation(value) => value.write_to(writer)?,
            StateValue::Flip(value) => value.write_to(writer)?,
            StateValue::Orientation(value) => value.write_to(writer)?,
            StateValue::Axis(value) => value.write_to(writer)?,
            StateValue::Coord2(value) => value.write_to(writer)?,
            StateValue::Coord3(value) => value.write_to(writer)?,
            StateValue::FaceFlags(value) => value.write_to(writer)?,
            StateValue::BitFlags8(value) => value.write_to(writer)?,
            StateValue::BitFlags16(value) => value.write_to(writer)?,
            StateValue::BitFlags32(value) => value.write_to(writer)?,
            StateValue::BitFlags64(value) => value.write_to(writer)?,
        } + 1)
    }
}

impl From<()> for StateValue {
    fn from(value: ()) -> Self {
        StateValue::Null
    }
}

impl From<i64> for StateValue {
    fn from(value: i64) -> Self {
        StateValue::Int(value)
    }
}

impl From<bool> for StateValue {
    fn from(value: bool) -> Self {
        StateValue::Bool(value)
    }
}

impl<S: StrToOwned> From<S> for StateValue {
    fn from(value: S) -> Self {
        StateValue::String(value.str_to_owned())
    }
}

impl From<Direction> for StateValue {
    fn from(value: Direction) -> Self {
        StateValue::Direction(value)
    }
}

impl From<Cardinal> for StateValue {
    fn from(value: Cardinal) -> Self {
        StateValue::Cardinal(value)
    }
}

impl From<Rotation> for StateValue {
    fn from(value: Rotation) -> Self {
        StateValue::Rotation(value)
    }
}

impl From<Flip> for StateValue {
    fn from(value: Flip) -> Self {
        StateValue::Flip(value)
    }
}

impl From<Orientation> for StateValue {
    fn from(value: Orientation) -> Self {
        StateValue::Orientation(value)
    }
}

impl From<Coord> for StateValue {
    fn from(value: Coord) -> Self {
        StateValue::Coord3(value.into())
    }
}

impl From<IVec3> for StateValue {
    fn from(value: IVec3) -> Self {
        StateValue::Coord3(value)
    }
}

impl From<(i32, i32, i32)> for StateValue {
    fn from(value: (i32, i32, i32)) -> Self {
        StateValue::Coord3(value.into())
    }
}

impl From<ChunkCoord> for StateValue {
    fn from(value: ChunkCoord) -> Self {
        StateValue::Coord2(value.into())
    }
}

impl From<IVec2> for StateValue {
    fn from(value: IVec2) -> Self {
        StateValue::Coord2(value)
    }
}

impl From<(i32, i32)> for StateValue {
    fn from(value: (i32, i32)) -> Self {
        StateValue::Coord2(value.into())
    }
}

impl From<Axis> for StateValue {
    fn from(value: Axis) -> Self {
        StateValue::Axis(value)
    }
}

impl From<FaceFlags> for StateValue {
    fn from(value: FaceFlags) -> Self {
        Self::FaceFlags(value)
    }
}

impl From<BitFlags8> for StateValue {
    fn from(value: BitFlags8) -> Self {
        Self::BitFlags8(value)
    }
}

impl From<BitFlags16> for StateValue {
    fn from(value: BitFlags16) -> Self {
        Self::BitFlags16(value)
    }
}

impl From<BitFlags32> for StateValue {
    fn from(value: BitFlags32) -> Self {
        Self::BitFlags32(value)
    }
}

impl From<BitFlags64> for StateValue {
    fn from(value: BitFlags64) -> Self {
        Self::BitFlags64(value)
    }
}

impl std::fmt::Display for StateValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateValue::Null => write!(f, "null"),
            StateValue::Int(value) => write!(f, "{value}"),
            StateValue::Bool(value) => write!(f, "{value}"),
            StateValue::String(value) => {
                write!(f, "\"")?;
                value.chars().try_for_each(|c| {
                    match c {
                        '\r' => write!(f, "\\r"),
                        '\n' => write!(f, "\\n"),
                        '\\' => write!(f, "\\\\"),
                        '"' => write!(f, "\\\""),
                        '\t' => write!(f, "\\t"),
                        '\0' => write!(f, "\\0"),
                        c => write!(f, "{c}"),
                    }
                })?;
                write!(f, "\"")
            },
            StateValue::Direction(direction) => match direction {
                Direction::NegX => write!(f, "NegX"),
                Direction::NegY => write!(f, "NegY"),
                Direction::NegZ => write!(f, "NegZ"),
                Direction::PosX => write!(f, "PosX"),
                Direction::PosY => write!(f, "PosY"),
                Direction::PosZ => write!(f, "PosZ"),
            },
            StateValue::Cardinal(cardinal) => match cardinal {
                Cardinal::West => write!(f, "West"),
                Cardinal::North => write!(f, "North"),
                Cardinal::East => write!(f, "East"),
                Cardinal::South => write!(f, "South"),
            },
            StateValue::Rotation(rotation) => {
                write!(f, "{rotation}")
            }
            StateValue::Flip(flip) => {
                write!(f, "{flip}")
            }
            StateValue::Orientation(orientation) => {
                write!(f, "{orientation}")
            }
            StateValue::Coord2(coord) => {
                write!(f, "({}, {})", coord.x, coord.y)
            }
            StateValue::Coord3(coord) => {
                write!(f, "({}, {}, {})", coord.x, coord.y, coord.z)
            }
            &StateValue::Axis(axis) => {
                write!(f, "Axis::{axis:?}")
            }
            &StateValue::FaceFlags(flags) => {
                write!(f, "{flags}")
            }
            &StateValue::BitFlags8(flags) => write!(f, "{flags}"),
            &StateValue::BitFlags16(flags) => write!(f, "{flags}"),
            &StateValue::BitFlags32(flags) => write!(f, "{flags}"),
            &StateValue::BitFlags64(flags) => write!(f, "{flags}"),
        }
    }
}