#![recursion_limit = "512"]
use std::io::{
    Read, Write,
    Seek,
};

use crate::core::error::{Error, Result};

pub fn write_zeros<W: Write>(writer: &mut W, count: u64) -> Result<u64> {
    const ZEROS: [u8; 4096] = [0; 4096];
    let mut count = count;
    // If we don't use >=,  we can optimize for the case where count is a multiple of 4096.
    while count > 4096 {
        writer.write_all(&ZEROS)?;
        count -= 4096;
    }
    writer.write_all(&ZEROS[0..count as usize])?;
    Ok(count)
}

pub trait Readable: Sized {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self>;
}


pub trait Writeable {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64>;
}

// pub trait ReaderExt: Read + Sized {
//     fn read_value<T: Readable>(&mut self) -> Result<T>;
// }

// pub trait WriterExt: Write + Sized {
//     fn write_value<T: Writeable>(&mut self, value: &T) -> Result<u64>;
// }

// impl<R: Read + Sized> ReaderExt for R {
//     fn read_value<T: Readable>(&mut self) -> Result<T> {
//         T::read_from(self)
//     }
// }

// impl<W: Write + Sized> WriterExt for W {
//     fn write_value<T: Writeable>(&mut self, value: &T) -> Result<u64> {
//         value.write_to(self)
//     }
// }

use crate::{core::{math::{bit::{BitFlags128, BitFlags16, BitFlags32, BitFlags64, BitFlags8}, coordmap::Rotation}, voxel::{axis::Axis, direction::Direction, rendering::color::{Rgb, Rgba}, tag::{Array, Byte, NonByte, Tag}}}, for_each_int_type};
use bevy::math::*;
use bytemuck::NoUninit;
use hashbrown::HashMap;
use rollgrid::{rollgrid2d::Bounds2D, rollgrid3d::Bounds3D};

use super::*;

const MAX_LEN: usize = 0xFFFFFF;

macro_rules! num_io {
    ($type:ty) => {
        impl Readable for $type {
            #[must_use]
            fn read_from<R: Read + Sized>(reader: &mut R) -> Result<Self> {
                let mut reader = reader;
                let mut buffer = [0u8; std::mem::size_of::<$type>()];
                reader.read_exact(&mut buffer)?;
                let value = <$type>::from_be_bytes(buffer);
                Ok(value)
            }
        }

        impl Writeable for $type {
            #[must_use]
            fn write_to<W: Write + Sized>(&self, writer: &mut W) -> Result<u64> {
                let mut writer = writer;
                let bytes = self.to_be_bytes();
                writer.write_all(&bytes)?;
                Ok(std::mem::size_of::<$type>() as u64)
            }
        }
    };
}

for_each_int_type!(num_io);

impl Readable for bool {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer)?;
        Ok(buffer[0] != 0)
    }
}

impl Writeable for bool {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        let bytes = [*self as u8];
        writer.write_all(&bytes)?;
        Ok(1)
    }
}

impl Readable for BitFlags8 {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer)?;
        Ok(BitFlags8(buffer[0]))
    }
}

impl Writeable for BitFlags8 {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        let buffer = [self.0];
        writer.write_all(&buffer)?;
        Ok(1)
    }
}

impl Readable for BitFlags16 {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        Ok(Self(u16::read_from(reader)?))
    }
}

impl Writeable for BitFlags16 {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        self.0.write_to(writer)
    }
}

impl Readable for BitFlags32 {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        Ok(Self(u32::read_from(reader)?))
    }
}

impl Writeable for BitFlags32 {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        self.0.write_to(writer)
    }
}

impl Readable for BitFlags64 {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        Ok(Self(u64::read_from(reader)?))
    }
}

impl Writeable for BitFlags64 {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        self.0.write_to(writer)
    }
}

impl Readable for BitFlags128 {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        Ok(Self(u128::read_from(reader)?))
    }
}

impl Writeable for BitFlags128 {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        self.0.write_to(writer)
    }
}

impl Readable for f32 {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        let mut buffer = [0u8; std::mem::size_of::<Self>()];
        reader.read_exact(&mut buffer)?;
        let value = Self::from_be_bytes(buffer);
        Ok(value)
    }
}

impl Writeable for f32 {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        let bytes = self.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(std::mem::size_of::<Self>() as u64)
    }
}

impl Readable for f64 {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        let mut buffer = [0u8; std::mem::size_of::<Self>()];
        reader.read_exact(&mut buffer)?;
        let value = Self::from_be_bytes(buffer);
        Ok(value)
    }
}

impl Writeable for f64 {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        let bytes = self.to_be_bytes();
        writer.write_all(&bytes)?;
        Ok(std::mem::size_of::<Self>() as u64)
    }
}

impl Readable for Direction {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        let dir: u8 = u8::read_from(reader)?;
        // NegX = 4,
        // NegY = 3,
        // NegZ = 5,
        // PosX = 1,
        // PosY = 0,
        // PosZ = 2,
        const NEGX: u8 = Direction::NegX as u8;
        const NEGY: u8 = Direction::NegY as u8;
        const NEGZ: u8 = Direction::NegZ as u8;
        const POSX: u8 = Direction::PosX as u8;
        const POSY: u8 = Direction::PosY as u8;
        const POSZ: u8 = Direction::PosZ as u8;
        use Direction::*;
        Ok(match dir {
            NEGX => NegX,
            NEGY => NegY,
            NEGZ => NegZ,
            POSX => PosX,
            POSY => PosY,
            POSZ => PosZ,
            _ => return Err(Error::InvalidBinaryFormat),
        })
    }
}

impl Writeable for Direction {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        (*self as u8).write_to(writer)
    }
}

impl Readable for Rotation {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        Ok(Rotation(u8::read_from(reader)?))
    }
}

impl Writeable for Rotation {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        self.0.write_to(writer)
    }
}

impl Readable for Axis {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        let axis: u8 = u8::read_from(reader)?;
        const X: u8 = Axis::X as u8;
        const Y: u8 = Axis::Y as u8;
        const Z: u8 = Axis::Z as u8;
        Ok(match axis {
            X => Axis::X,
            Y => Axis::Y,
            Z => Axis::Z,
            _ => return Err(Error::InvalidBinaryFormat)
        })
    }
}

impl Writeable for Axis {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        (*self as u8).write_to(writer)
    }
}

impl Readable for Rgb {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        let mut rgb = [0u8; 3];
        reader.read_exact(&mut rgb)?;
        let [r,g,b] = rgb;
        Ok(Rgb::new(r,g,b))
    }
}

impl Writeable for Rgb {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        let buf = [self.r, self.g, self.b];
        writer.write_all(&buf)?;
        Ok(3)
    }
}

impl Readable for Rgba {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut reader = reader;
        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        let [r, g, b, a] = buffer;
        Ok(Rgba::new(r, g, b, a))
    }
}

impl Writeable for Rgba {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut writer = writer;
        let buf = [self.r, self.g, self.b, self.a];
        writer.write_all(&buf)?;
        Ok(4)
    }
}

impl Readable for IVec2 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(ivec2(i32::read_from(reader)?, i32::read_from(reader)?))
    }
}

impl Writeable for IVec2 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.x.write_to(writer)?;
        self.y.write_to(writer)?;
        Ok(4 * 2)
    }
}

impl Readable for IVec3 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(ivec3(i32::read_from(reader)?, i32::read_from(reader)?, i32::read_from(reader)?))
    }
}

impl Writeable for IVec3 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.x.write_to(writer)?;
        self.y.write_to(writer)?;
        self.z.write_to(writer)?;
        Ok(4 * 3)
    }
}

impl Readable for IVec4 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(ivec4(
            i32::read_from(reader)?,
            i32::read_from(reader)?,
            i32::read_from(reader)?,
            i32::read_from(reader)?,
        ))
    }
}

impl Writeable for IVec4 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.x.write_to(writer)?;
        self.y.write_to(writer)?;
        self.z.write_to(writer)?;
        self.w.write_to(writer)?;
        Ok(4 * 4)
    }
}

impl Readable for Vec2 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(vec2(f32::read_from(reader)?, f32::read_from(reader)?))
    }
}

impl Writeable for Vec2 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.x.write_to(writer)?;
        self.y.write_to(writer)?;
        Ok(4 * 2)
    }
}

impl Readable for Vec3 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(vec3(
            f32::read_from(reader)?,
            f32::read_from(reader)?,
            f32::read_from(reader)?
        ))
    }
}

impl Writeable for Vec3 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.x.write_to(writer)?;
        self.y.write_to(writer)?;
        self.z.write_to(writer)?;
        Ok(4 * 3)
    }
}

impl Readable for Vec4 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(vec4(
            f32::read_from(reader)?,
            f32::read_from(reader)?,
            f32::read_from(reader)?,
            f32::read_from(reader)?
        ))
    }
}

impl Writeable for Vec4 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.x.write_to(writer)?;
        self.y.write_to(writer)?;
        self.z.write_to(writer)?;
        self.w.write_to(writer)?;
        Ok(4 * 4)
    }
}

impl Readable for Mat2 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        // 0 2
        // 1 3
        let mut data = [f32::NAN; 4];
        for i in 0..4 {
            data[i] = f32::read_from(reader)?;
        }
        Ok(mat2(vec2(data[0], data[1]), vec2(data[2], data[3])))
    }
}

impl Writeable for Mat2 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        let data = self.to_cols_array();
        for i in 0..4 {
            data[i].write_to(writer)?;
        }
        Ok(4*4)
    }
}

impl Readable for Mat3 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut cols = [f32::NAN; 9];
        for i in 0..9 {
            cols[i] = f32::read_from(reader)?;
        }
        Ok(mat3(
            vec3(
                cols[0],
                cols[1],
                cols[2]
            ),
            vec3(
                cols[3],
                cols[4],
                cols[5]
            ),
            vec3(
                cols[6],
                cols[7],
                cols[8]
            )
        ))
    }
}

impl Writeable for Mat3 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        let col = self.col(0);
        col.x.write_to(writer)?;
        col.y.write_to(writer)?;
        col.z.write_to(writer)?;
        let col = self.col(1);
        col.x.write_to(writer)?;
        col.y.write_to(writer)?;
        col.z.write_to(writer)?;
        let col = self.col(2);
        col.x.write_to(writer)?;
        col.y.write_to(writer)?;
        col.z.write_to(writer)?;
        Ok(4 * 3*3)
    }
}

impl Readable for Mat4 {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut cols = [f32::NAN; 16];
        for i in 0..16 {
            cols[i] = f32::read_from(reader)?;
        }
        Ok(mat4(
            vec4(
                cols[0],
                cols[1],
                cols[2],
                cols[3]
            ),
            vec4(
                cols[4],
                cols[5],
                cols[6],
                cols[7]
            ),
            vec4(
                cols[8],
                cols[9],
                cols[10],
                cols[11]
            ),
            vec4(
                cols[12],
                cols[13],
                cols[14],
                cols[15]
            )
        ))
    }
}

impl Writeable for Mat4 {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        let col = self.col(0);
        col.x.write_to(writer)?;
        col.y.write_to(writer)?;
        col.z.write_to(writer)?;
        col.w.write_to(writer)?;
        let col = self.col(1);
        col.x.write_to(writer)?;
        col.y.write_to(writer)?;
        col.z.write_to(writer)?;
        col.w.write_to(writer)?;
        let col = self.col(2);
        col.x.write_to(writer)?;
        col.y.write_to(writer)?;
        col.z.write_to(writer)?;
        col.w.write_to(writer)?;
        let col = self.col(3);
        col.x.write_to(writer)?;
        col.y.write_to(writer)?;
        col.z.write_to(writer)?;
        col.w.write_to(writer)?;
        Ok(4 * 4*4)
    }
}

impl Readable for Quat {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(quat(
            f32::read_from(reader)?,
            f32::read_from(reader)?,
            f32::read_from(reader)?,
            f32::read_from(reader)?
        ))
    }
}

impl Writeable for Quat {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.x.write_to(writer)?;
        self.y.write_to(writer)?;
        self.z.write_to(writer)?;
        self.w.write_to(writer)?;
        Ok(4 * 4)
    }
}

impl Readable for Bounds2D {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(Bounds2D {
            min: (i32::read_from(reader)?,i32::read_from(reader)?),
            max: (i32::read_from(reader)?,i32::read_from(reader)?)
        })
    }
}

impl Writeable for Bounds2D {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.min.0.write_to(writer)?;
        self.min.1.write_to(writer)?;
        self.max.0.write_to(writer)?;
        self.max.1.write_to(writer)?;
        Ok(4 * 2 * 2)
    }
}

impl Readable for Bounds3D {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(Bounds3D {
            min: (i32::read_from(reader)?,i32::read_from(reader)?,i32::read_from(reader)?),
            max: (i32::read_from(reader)?,i32::read_from(reader)?,i32::read_from(reader)?)
        })
    }
}

impl Writeable for Bounds3D {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.min.0.write_to(writer)?;
        self.min.1.write_to(writer)?;
        self.min.2.write_to(writer)?;
        self.max.0.write_to(writer)?;
        self.max.1.write_to(writer)?;
        self.max.2.write_to(writer)?;
        Ok(4 * 3 * 2)
    }
}

impl Readable for String {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        let bytes = read_bytes(reader, length as usize)?;
        Ok(String::from_utf8(bytes)?)
    }
}

impl Writeable for String {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::StringTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        write_bytes(writer, self.as_bytes())?;
        Ok(self.len() as u64 + 3)
    }
}

impl<T: Readable + NonByte> Readable for Vec<T> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        let mut result = Vec::new();
        for _ in 0..length {
            result.push(T::read_from(reader)?);
        }
        Ok(result)
    }
}

impl<T: Writeable + NonByte> Writeable for Vec<T> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::ArrayTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        let mut length = 0;
        for i in 0..self.len() {
            length + self[i].write_to(writer)?;
        }
        Ok(length + 3)
    }
}

impl<T: Readable> Readable for Box<T> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        Ok(Box::new(T::read_from(reader)?))
    }
}

impl<T: Writeable> Writeable for Box<T> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        self.as_ref().write_to(writer)
    }
}

impl Readable for Vec<bool> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        let bytes = read_bytes(reader, length as usize)?;
        Ok(bytes.into_iter().map(|b| b != 0).collect())
    }
}

impl Writeable for Vec<bool> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::ArrayTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        let bytes: &[u8] = bytemuck::cast_slice(&self.as_slice());
        writer.write_all(&bytes)?;
        Ok(self.len() as u64 + 3)
    }
}

impl Readable for Vec<BitFlags8> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        let bytes = read_bytes(reader, length as usize)?;
        Ok(bytes.into_iter().map(|b| BitFlags8(b)).collect())
    }
}

impl Writeable for Vec<BitFlags8> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::ArrayTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        let bytes: &[u8] = bytemuck::cast_slice(&self.as_slice());
        writer.write_all(&bytes)?;
        Ok(self.len() as u64 + 3)
    }
}

impl Readable for Vec<u8> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        read_bytes(reader, length as usize)
    }
}

impl Writeable for Vec<u8> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::ArrayTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        writer.write_all(self.as_slice())?;
        Ok(self.len() as u64 + 3)
    }
}

impl Readable for Vec<i8> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        Ok(read_bytes(reader, length as usize)?.into_iter().map(|b| b as i8).collect())
    }
}

impl Writeable for Vec<i8> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::ArrayTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        writer.write_all(bytemuck::cast_slice(self.as_slice()))?;
        Ok(self.len() as u64 + 3)
    }
}

impl Readable for Vec<Direction> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        let bytes = read_bytes(&mut reader, length as usize)?;
        const NEGX: u8 = Direction::NegX as u8;
        const NEGY: u8 = Direction::NegY as u8;
        const NEGZ: u8 = Direction::NegZ as u8;
        const POSX: u8 = Direction::PosX as u8;
        const POSY: u8 = Direction::PosY as u8;
        const POSZ: u8 = Direction::PosZ as u8;
        Ok(bytes.into_iter().map(|b| match b {
            NEGX => Direction::NegX,
            NEGY => Direction::NegY,
            NEGZ => Direction::NegZ,
            POSX => Direction::PosX,
            POSY => Direction::PosY,
            POSZ => Direction::PosZ,
            _ => panic!("Invalid binary format for Direction (sorry for the panic)"),
        }).collect())
    }
}

impl Writeable for Vec<Direction> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::ArrayTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        writer.write_all(bytemuck::cast_slice(self.as_slice()))?;
        Ok(self.len() as u64 + 3)
    }
}

impl Readable for Vec<Rotation> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        let bytes = read_bytes(reader, length as usize)?;
        Ok(bytes.into_iter().map(|b| Rotation(b)).collect())
    }
}

impl Writeable for Vec<Rotation> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::ArrayTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        writer.write_all(bytemuck::cast_slice(self.as_slice()))?;
        Ok(self.len() as u64 + 3)
    }
}

impl Readable for Vec<Axis> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let length = u32::from_be_bytes(buf);
        let bytes = read_bytes(reader, length as usize)?;
        Ok(bytes.into_iter().map(|b| match b {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            _ => panic!("Invalid binary format for Axis")
        }).collect())
    }
}

impl Writeable for Vec<Axis> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        if self.len() > MAX_LEN {
            return Err(Error::ArrayTooLong);
        }
        let buf = (self.len() as u32).to_be_bytes();
        writer.write_all(&buf[1..4])?;
        writer.write_all(bytemuck::cast_slice(self.as_slice()))?;
        Ok(self.len() as u64 + 3)
    }
}

impl Readable for hashbrown::HashMap<String, Tag> {
    fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
        // read name, read tag, use 255 as Stop
        let mut map = HashMap::new();
        loop {
            let id: u8 = u8::read_from(reader)?;
            // Stop
            if id == 255 {
                break;
            }
            let name: String = String::read_from(reader)?;
            let tag: Tag = Tag::read_from(reader)?;
            map.insert(name, tag);
        }
        Ok(map)
    }
}

impl Writeable for hashbrown::HashMap<String, Tag> {
    fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
        let size = self.iter().try_fold(0u64, |mut size, (name, tag)| {
            size += tag.id().write_to(writer)?;
            size += name.write_to(writer)?;
            size += tag.write_to(writer)?;
            Result::Ok(size)
        })?;
        255u8.write_to(writer)?;
        // writer.write_value(&255u8)?;
        Ok(size + 1)
    }
}

/// Reads an exact number of bytes from a reader, returning them as a [Vec].
fn read_bytes<R: Read>(reader: &mut R, length: usize) -> Result<Vec<u8>> {
    let mut reader = reader;
    let mut buf: Vec<u8> = vec![0u8; length];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// Writes a byte slice to a writer, returning the number of bytes that were written.
fn write_bytes<W: Write>(writer: &mut W, data: &[u8]) -> Result<usize> {
    let mut writer = writer;
    Ok(writer.write_all(data).map(|_| data.len())?)
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::{BufReader, BufWriter, Cursor, SeekFrom}};
    use super::*;

    use hashbrown::HashMap;

    use super::voxel::tag::{Array, Tag};

    #[test]
    fn io_test() -> Result<()> {
        let tag = Tag::from(HashMap::from([
            ("axis".to_owned(), Tag::Axis(Axis::Y)),
            ("".to_owned(), Tag::from(BitFlags128(u128::MAX))),
            ("pi".to_owned(), Tag::from(3.14)),
            ("nested".to_owned(), Tag::from(HashMap::new())),
            ("array".to_owned(), Tag::from(Array::I64(vec![1,2,3,4])))
        ]));
        let mut cursor = Cursor::new(Vec::new());
        tag.write_to(&mut cursor)?;
        // writer.write_value(&tag)?;
        cursor.seek(SeekFrom::Start(0))?;
        let read_tag = Tag::read_from(&mut cursor)?;
        assert_eq!(tag, read_tag);
        Ok(())
    }
}