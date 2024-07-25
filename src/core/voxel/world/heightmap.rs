#![allow(unused)]
use crate::core::voxel::coord::Coord;
use crate::prelude::{Readable, Writeable};

use super::*;
use super::world::WORLD_HEIGHT;
use crate::core::error::*;

// Assuming world height of 640, this has a size of 80 bytes.
pub struct HeightmapColumn {
    masks: [u64; Self::SECTION_COUNT]
}

impl Default for HeightmapColumn {
    fn default() -> Self {
        Self {
            masks: [0; Self::SECTION_COUNT]
        }
    }
}

impl HeightmapColumn {
    const SECTION_COUNT: usize = WORLD_HEIGHT / 64 + ((WORLD_HEIGHT % 64 > 0) as usize);
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, index: usize) -> bool {
        let sub_index = index / 64;
        let bit_index = index & 0b111111;
        self.masks[sub_index] & (1 << bit_index) != 0
    }

    pub fn set(&mut self, index: usize, value: bool) -> bool {
        let sub_index = index / 64;
        let bit_index = index & 0b111111;
        let old = self.masks[sub_index] & (1 << bit_index) != 0;
        if old != value {
            self.masks[sub_index] = if value {
                self.masks[sub_index] | (1 << bit_index)
            } else {
                self.masks[sub_index] & !(1 << bit_index)
            }
        }
        old
    }

    /// Calculates the height by iterating through the bitmasks in the column and counting the leading zeros.
    pub fn height(&self) -> usize {
        for i in (0..self.masks.len()).rev() {
            if self.masks[i] == 0 {
                continue;
            }
            let height = 64 - self.masks[i].leading_zeros();
            if height != 0 {
                return height as usize + i * 64;
            }
        }
        0
    }

    pub fn read_from<R: std::io::Read>(&mut self, reader: &mut R) -> Result<()> {
        self.masks.iter_mut().try_for_each(|mask| {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf)?;
            // We're using little endian bytes so that the bits
            // are laid out in memory sequentially.
            *mask = u64::from_le_bytes(buf);
            Result::Ok(())
        })
    }

    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<u64> {
        self.masks.iter().try_fold(0u64, |length, &mask| {
            let bytes = mask.to_le_bytes();
            writer.write_all(&bytes)?;
            Result::Ok(length + 8)
        })
    }
}

pub struct Heightmap {
    columns: Box<[HeightmapColumn]>,
    heightmap: Box<[u16]>,
}

impl Default for Heightmap {
    fn default() -> Self {
        Self {
            columns: (0..256).map(|_| HeightmapColumn::new()).collect(),
            heightmap: (0..256).map(|_| 0).collect(),
        }
    }
}

impl Heightmap {
    /* Size Table
    |--------|---------------|
    | Height | SECTION_COUNT |
    |--------|---------------|
    |    128 | 1             |
    |    256 | 2             |
    |    512 | 4             |
    |   1024 | 8             |
    |   2048 | 16            |
    |   4096 | 32            |
    |   8129 | 64            |
    |--------|---------------|*/
    const SECTION_COUNT: usize = (WORLD_HEIGHT / 64) + (((WORLD_HEIGHT % 64) > 0) as usize);
    pub const HEIGHT: usize = Self::SECTION_COUNT * 64;
    pub const MAX: i32 = Self::HEIGHT as i32 - 1;
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, coord: Coord) -> bool {
        let x = coord.x & 0xF;
        let z = coord.z & 0xF;
        let col_index = x | z << 4;
        self.columns[col_index as usize].get(coord.y as usize)
    }

    pub fn set(&mut self, coord: Coord, value: bool) -> bool {
        let x = coord.x & 0xF;
        let z = coord.z & 0xF;
        let col_index = (x | z << 4) as usize;
        let height = self.heightmap[col_index];
        let old = self.columns[col_index].set(coord.y as usize, value);
        if value != old {
            if value {
                if coord.y + 1 > height as i32 {
                    self.heightmap[col_index] = (coord.y + 1) as u16;
                }
            } else {
                if coord.y + 1 == height as i32 {
                    let new_height = self.columns[col_index].height();
                    self.heightmap[col_index] = new_height as u16;
                }
            }
        }
        old
    }

    pub fn height(&self, x: i32, z: i32) -> i32 {
        let x = x & 0xF;
        let z = z & 0xF;
        let col_index = x | z << 4;
        let height = self.heightmap[col_index as usize] as i32;
        height
    }

    pub fn read_from<R: std::io::Read>(&mut self, reader: &mut R) -> Result<()> {
        // First read the heightmap, then read the columns
        self.heightmap.iter_mut().try_for_each(|height| {
            *height = u16::read_from(reader)?;
            Result::Ok(())
        })?;
        self.columns.iter_mut().try_for_each(|col| {
            col.read_from(reader)
        })?;
        Ok(())
    }

    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<u64> {
        let length = self.heightmap.iter().try_fold(0u64, |length, &height| {
            Result::Ok(length + height.write_to(writer)?)
        })?;
        let length = self.columns.iter().try_fold(length, |length, col| {
            Result::Ok(length + col.write_to(writer)?)
        })?;
        Ok(length)
    }
}

#[cfg(test)]
mod testing_sandbox {
    use super::*;
    #[test]
    fn sandbox() {
        let val = 0x04030201u32;
        let bytes = val.to_le_bytes();
        println!("{bytes:?}");
    }
}