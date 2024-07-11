use crate::core::voxel::coord::Coord;

use super::world::WORLD_HEIGHT;

pub struct HeightmapColumn {
    masks: [u128; Self::SECTION_COUNT]
}

impl Default for HeightmapColumn {
    fn default() -> Self {
        Self {
            masks: [0; Self::SECTION_COUNT]
        }
    }
}

impl HeightmapColumn {
    const SECTION_COUNT: usize = WORLD_HEIGHT / 128 + ((WORLD_HEIGHT % 128 > 0) as usize);
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, index: usize) -> bool {
        let sub_index = index / 128;
        let bit_index = index & 0b1111111;
        self.masks[sub_index] & (1 << bit_index) != 0
    }

    pub fn set(&mut self, index: usize, value: bool) -> bool {
        let sub_index = index / 128;
        let bit_index = index & 0b1111111;
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

    pub fn height(&self) -> usize {
        for i in (0..self.masks.len()).rev() {
            if self.masks[i] == 0 {
                continue;
            }
            let height = 128 - self.masks[i].leading_zeros();
            if height != 0 {
                return height as usize + i * 128;
            }
        }
        0
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
    const  WORLD_HEIGHT: usize = 640;
    const SECTION_COUNT: usize = (Heightmap::WORLD_HEIGHT / 128) + (((Heightmap::WORLD_HEIGHT % 128) > 0) as usize);
    pub const HEIGHT: usize = Self::SECTION_COUNT * 128;
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
        old
    }

    pub fn height(&self, x: i32, z: i32) -> i32 {
        let x = x & 0xF;
        let z = z & 0xF;
        let col_index = x | z << 4;
        self.heightmap[col_index as usize] as i32
    }
}