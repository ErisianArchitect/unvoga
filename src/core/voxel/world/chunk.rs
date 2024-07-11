use crate::core::voxel::{blocks::StateRef, coord::Coord};

use super::{heightmap::Heightmap, world::WORLD_HEIGHT};

// 20480 bytes
pub struct Section {
    blocks: Box<[StateRef]>,
    block_light: Box<[u8]>,
    sky_light: Box<[u8]>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            blocks: (0..4096).map(|_| StateRef::AIR).collect(),
            block_light: (0..2048).map(|_| 0).collect(),
            sky_light: (0..2048).map(|_| 0).collect(),
        }
    }

    fn index(coord: Coord) -> usize {
        let x = (coord.x & 0xF) as usize;
        let y = (coord.y & 0xF) as usize;
        let z = (coord.z & 0xF) as usize;
        x | z << 4 | y << 8
    }

    pub fn get(&self, coord: Coord) -> StateRef {
        let index = Section::index(coord);
        self.blocks[index]
    }

    pub fn set(&mut self, coord: Coord, state: StateRef) -> StateRef {
        let index = Section::index(coord);
        let mut old = state;
        std::mem::swap(&mut self.blocks[index], &mut old);
        old
    }

    pub fn get_block_light(&self, coord: Coord) -> u8 {
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1) * 4;
        (self.block_light[mask_index] & (0xF << sub_index)) >> sub_index
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> u8 {
        let level = level.max(15);
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1);
        let other_index = (sub_index - 1) & 1;
        let other_shift = other_index * 4;
        let shift = sub_index * 4;
        let old = (self.block_light[mask_index] & (0xF << shift)) >> shift;
        let other = self.block_light[mask_index] & (0xF << other_shift);
        self.block_light[mask_index] = other | (level << shift);
        old
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1) * 4;
        (self.sky_light[mask_index] & (0xF << sub_index)) >> sub_index
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> u8 {
        let level = level.max(15);
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1);
        let other_index = (sub_index - 1) & 1;
        let other_shift = other_index * 4;
        let shift = sub_index * 4;
        let old = (self.sky_light[mask_index] & (0xF << shift)) >> shift;
        let other = self.sky_light[mask_index] & (0xF << other_shift);
        self.sky_light[mask_index] = other | (level << shift);
        old
    }
}

pub struct Chunk {
    sections: Box<[Option<Section>]>,
    heightmap: Heightmap,
    /// The offset block coordinate.
    offset: Coord,
}

impl Chunk {
    const SECTION_COUNT: usize = WORLD_HEIGHT / 16;
    pub fn new(offset: Coord) -> Self {
        Self {
            sections: (0..Self::SECTION_COUNT).map(|_| None).collect(),
            heightmap: Heightmap::new(),
            offset,
        }
    }

    pub fn get(&self, coord: Coord) -> StateRef {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = coord.y as usize / 16;
        if let Some(section) = &self.sections[section_index] {
            section.get(coord)
        } else {
            StateRef::AIR
        }
    }

    pub fn set(&mut self, coord: Coord, value: StateRef) -> StateRef {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = coord.y as usize / 16;
        let nonair = value != StateRef::AIR;
        self.heightmap.set(coord, nonair);
        if self.sections[section_index].is_none() {
            if value != StateRef::AIR {
                self.sections[section_index] = Some(Section::new());
            } else {
                return StateRef::AIR;
            }
        }
        let Some(section) = &mut self.sections[section_index] else {
            unreachable!()
        };
        section.set(coord, value)
    }

    pub fn get_block_light(&self, coord: Coord) -> u8 {
        let section_index = coord.y as usize / 16;
        if let Some(section) = &self.sections[section_index] {
            section.get_block_light(coord)
        } else {
            0
        }
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        let section_index = coord.y as usize / 16;
        if let Some(section) = &self.sections[section_index] {
            section.get_sky_light(coord)
        } else {
            0
        }
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> u8 {
        let section_index = coord.y as usize / 16;
        if self.sections[section_index].is_none() {
            if level > 0 {
                self.sections[section_index] = Some(Section::new());
            } else {
                return 0;
            }
        }
        let Some(section) = &mut self.sections[section_index] else {
            unreachable!()
        };
        section.set_block_light(coord, level)
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> u8 {
        let section_index = coord.y as usize / 16;
        if self.sections[section_index].is_none() {
            if level > 0 {
                self.sections[section_index] = Some(Section::new());
            } else {
                return 0;
            }
        }
        let Some(section) = &mut self.sections[section_index] else {
            unreachable!()
        };
        section.set_sky_light(coord, level)
    }

    pub fn height(&self, x: i32, z: i32) -> i32 {
        self.heightmap.height(x, z)
    }
}

#[test]
fn wrap_test() {
    let neg = -235i32;
    let wrap = neg & 0xF;
    println!("{wrap} -> {}", neg.rem_euclid(16));
}