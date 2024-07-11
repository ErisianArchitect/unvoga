
use std::collections::VecDeque;

use rollgrid::RollGrid2D;

use crate::core::voxel::{blocks::StateRef, coord::Coord, engine::VoxelEngine};

use super::chunk::{Chunk, LightChange, Section, StateChange};

// Make sure this value is always a multiple of 16 and
// preferably a multiple of 128.
pub const WORLD_HEIGHT: usize = 640;
pub const WORLD_SIZE_MAX: usize = 20;
/// The pad size is the added chunk width for light updates.
/// If WORLD_SIZE_MAX is the range that is visible to the player,
/// PADDED_WORLD_SIZE_MAX is the range that  has light updates (since
/// light updates can span multple chunks).
pub const WORLD_SIZE_MAX_PAD: usize = 2;
pub const PADDED_WORLD_SIZE_MAX: usize = WORLD_SIZE_MAX + WORLD_SIZE_MAX_PAD;

pub struct World {
    chunks: RollGrid2D<Chunk>,
    dirty_sections: Vec<Coord>,
}

impl World {
    pub fn new(size: usize, center: (i32, i32)) -> Self {
        if size > PADDED_WORLD_SIZE_MAX {
            panic!("Size greater than {PADDED_WORLD_SIZE_MAX} (PADDED_WORLD_SIZE_MAX)");
        }
        let full_size = size * 2;
        let size = size as i32;
        let (x, z) = center;
        let (offx, offz) = (x - 8, z - 8);
        let size_offset = (size - 1) * 16;
        let (snapx, snapz) = (
            offx & !0xF,
            offz & !0xF
        );
        Self {
            chunks: RollGrid2D::new_with_init(full_size, full_size, (snapx / 16, snapz / 16), |(x, z): (i32, i32)| {
                
                Some(Chunk::new(Coord::new(x * 16, 0, z * 16)))
            }),
            dirty_sections: Vec::new(),
        }
    }

    pub fn offset(&self) -> Coord {
        let grid_offset = self.chunks.offset();
        Coord::new(
            grid_offset.0 * 16,
            0,
            grid_offset.1 * 16
        )
    }

    pub fn get_section(&self, section_coord: Coord) -> Option<&Section> {
        if section_coord.y < 0 {
            return None;
        }
        let chunk = self.chunks.get((section_coord.x, section_coord.z))?;
        let y = section_coord.y - chunk.offset.y / 16;
        if y < 0 || y as usize >= chunk.sections.len() {
            return None;
        }
        Some(&chunk.sections[y as usize])
    }

    pub fn get_section_mut(&mut self, section_coord: Coord) -> Option<&mut Section> {
        if section_coord.y < 0 {
            return None;
        }
        let chunk = self.chunks.get_mut((section_coord.x, section_coord.z))?;
        let y = section_coord.y - chunk.offset.y / 16;
        if y < 0 || y as usize >= chunk.sections.len() {
            return None;
        }
        Some(&mut chunk.sections[y as usize])
    }

    pub fn get_block(&self, coord: Coord) -> StateRef {
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
            chunk.get(coord)
        } else {
            StateRef::AIR
        }
    }

    pub fn get_block_light(&self, coord: Coord) -> u8 {
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
            chunk.get_block_light(coord)
        } else {
            0
        }
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
            chunk.get_sky_light(coord)
        } else {
            0
        }
    }

    pub fn set_block(&mut self, coord: Coord, state: StateRef) -> StateRef {
        // let blocks = engine.blocks();
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.set(coord, state)
        } else {
            panic!("Out of bounds");
        };
        match change.change {
            StateChange::Unchanged => state,
            StateChange::Changed(old) => {
                state.block().on_place(self, coord, state);
                if change.marked_dirty {
                    let section_coord = coord.section_coord();
                    self.dirty_sections.push(section_coord);
                }
                old
            },
        }
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> LightChange {
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.set_block_light(coord, level)
        } else {
            panic!("Out of bounds");
        };
        if change.marked_dirty {
            let section_coord = coord.section_coord();
            self.dirty_sections.push(section_coord);
        }
        if change.change.new_max != change.change.old_max {
            let block = self.get_block(coord);
            block.block().light_updated(self, coord, change.change.old_max, change.change.new_max);
        }
        change.change
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> LightChange {
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.set_sky_light(coord, level)
        } else {
            panic!("Out of bounds");
        };
        if change.marked_dirty {
            let section_coord = coord.section_coord();
            self.dirty_sections.push(section_coord);
        }
        if change.change.new_max != change.change.old_max {
            let block = self.get_block(coord);
            block.block().light_updated(self, coord, change.change.old_max, change.change.new_max);
        }
        change.change
    }
}

#[test]
fn center_test() {
    let size: i32 = 16;
    let x: i32 = 16*16-8;
    let offx = x - 8;
    let size_offset = (size - 1) * 16;
    let snap = offx - offx.rem_euclid(16);
    let result = snap - size_offset;
    println!("  Snap: {snap}");
    println!("Result: {result}");
}