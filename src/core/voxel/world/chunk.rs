use bevy::prelude::state_changed;

use crate::core::voxel::{blocks::StateRef, coord::Coord};

use super::{heightmap::Heightmap, world::WORLD_HEIGHT};

pub struct SectionBlocks {
    blocks: Box<[StateRef]>
}

impl SectionBlocks {
    pub fn new() -> Self {
        Self {
            blocks: (0..4096).map(|_| StateRef::AIR).collect()
        }
    }
}

fn make_empty_section_blocks() -> Box<[StateRef]> {
    (0..4096).map(|_| StateRef::AIR).collect()
}

fn make_empty_section_light() -> Box<[u8]> {
    (0..2048).map(|_| 0).collect()
}

// 20480 bytes
pub struct Section {
    blocks: Option<Box<[StateRef]>>,
    block_light: Option<Box<[u8]>>,
    sky_light: Option<Box<[u8]>>,
    block_count: u16,
    block_light_count: u16,
    sky_light_count: u16,
}

impl Section {
    pub fn new() -> Self {
        Self {
            blocks: None,
            block_light: None,
            sky_light: None,
            block_count: 0,
            block_light_count: 0,
            sky_light_count: 0,
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
        if let Some(blocks) = &self.blocks {
            blocks[index]
        } else {
            StateRef::AIR
        }
    }

    pub fn set(&mut self, coord: Coord, state: StateRef) -> StateRef {
        let index = Section::index(coord);
        if self.blocks.is_none() {
            if !state.is_air() {
                self.blocks = Some(make_empty_section_blocks());
            } else {
                return StateRef::AIR;
            }
        }
        let Some(blocks) = &mut self.blocks else {
            panic!("Should have been valid");
        };
        let mut old = state;
        std::mem::swap(&mut blocks[index], &mut old);
        if state != old {
            if !state.is_air() && old.is_air() {
                self.block_count += 1;
            } else if state.is_air() && !old.is_air() {
                self.block_count -= 1;
                if self.block_count == 0 {
                    self.blocks = None;
                }
            }
        }
        old
    }

    pub fn get_block_light(&self, coord: Coord) -> u8 {
        if let Some(block_light) = &self.block_light {
            let index = Section::index(coord);
            let mask_index = index / 2;
            let sub_index = (index & 1) * 4;
            (block_light[mask_index] & (0xF << sub_index)) >> sub_index
        } else {
            0
        }
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> u8 {
        let level = level.min(15);
        if self.block_light.is_none() {
            if level != 0 {
                self.block_light = Some(make_empty_section_light());
            } else {
                return 0;
            }
        }
        let Some(block_light) = &mut self.block_light else {
            panic!("Should be valid");
        };
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1);
        let other_index = (sub_index - 1) & 1;
        let other_shift = other_index * 4;
        let shift = sub_index * 4;
        let old = (block_light[mask_index] & (0xF << shift)) >> shift;
        let other = block_light[mask_index] & (0xF << other_shift);
        block_light[mask_index] = other | (level << shift);

        if level != 0 && old == 0 {
            self.block_light_count += 1;
        } else if level == 0 && old != 0 {
            self.block_light_count -= 1;
            if self.block_light_count == 0 {
                self.block_light = None;
            }
        }

        old
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        if let Some(sky_light) = &self.sky_light {
            let index = Section::index(coord);
            let mask_index = index / 2;
            let sub_index = (index & 1) * 4;
            (sky_light[mask_index] & (0xF << sub_index)) >> sub_index
        } else {
            0
        }
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> u8 {
        let level = level.min(15);
        if self.sky_light.is_none() {
            if level != 0 {
                self.sky_light = Some(make_empty_section_light());
            } else {
                return 0;
            }
        }
        let Some(sky_light) = &mut self.sky_light else {
            panic!("Should be valid");
        };
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1);
        let other_index = (sub_index - 1) & 1;
        let other_shift = other_index * 4;
        let shift = sub_index * 4;
        let old = (sky_light[mask_index] & (0xF << shift)) >> shift;
        let other = sky_light[mask_index] & (0xF << other_shift);
        sky_light[mask_index] = other | (level << shift);
        if level != 0 && old == 0 {
            self.sky_light_count += 1;
        } else if level == 0 && old != 0 {
            self.sky_light_count -= 1;
            if self.sky_light_count == 0 {
                self.sky_light = None;
            }
        }
        old
    }
}

pub struct Chunk {
    sections: Box<[Section]>,
    heightmap: Heightmap,
    /// The offset block coordinate.
    offset: Coord,
}

impl Chunk {
    const SECTION_COUNT: usize = WORLD_HEIGHT / 16;
    pub fn new(offset: Coord) -> Self {
        Self {
            sections: (0..Self::SECTION_COUNT).map(|_| Section::new()).collect(),
            heightmap: Heightmap::new(),
            offset,
        }
    }

    pub fn get(&self, coord: Coord) -> StateRef {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = coord.y as usize / 16;
        self.sections[section_index].get(coord)
    }

    pub fn set(&mut self, coord: Coord, value: StateRef) -> StateRef {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = coord.y as usize / 16;
        let nonair = value != StateRef::AIR;
        self.heightmap.set(coord, nonair);
        self.sections[section_index].set(coord, value)
    }

    pub fn get_block_light(&self, coord: Coord) -> u8 {
        let section_index = coord.y as usize / 16;
        self.sections[section_index].get_block_light(coord)
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        let section_index = coord.y as usize / 16;
        self.sections[section_index].get_sky_light(coord)
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> u8 {
        let section_index = coord.y as usize / 16;
        self.sections[section_index].set_block_light(coord, level)
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> u8 {
        let section_index = coord.y as usize / 16;
        self.sections[section_index].set_sky_light(coord, level)
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