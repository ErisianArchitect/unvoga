
use rollgrid::RollGrid2D;

use crate::core::voxel::{blocks::StateRef, coord::Coord, engine::VoxelEngine};

use super::chunk::{Chunk, LightChange};

// Make sure this value is always a multiple of 16 and
// preferably a multiple of 128.
pub const WORLD_HEIGHT: usize = 640;

pub struct World {
    chunks: RollGrid2D<Chunk>,
}

impl World {
    pub fn new(size: usize, center: (i32, i32)) -> Self {
        if size > 16 {
            panic!("Size greater than 16");
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
            })
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
        let old = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.set(coord, state)
        } else {
            panic!("Out of bounds");
        };
        state.block().on_place(self, coord, state);
        old
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> LightChange {
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.set_block_light(coord, level)
        } else {
            panic!("Out of bounds");
        };
        if change.new_max != change.old_max {
            let block = self.get_block(coord);
            block.block().light_updated(self, coord, change.old_max, change.new_max);
        }
        change
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> LightChange {
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.set_sky_light(coord, level)
        } else {
            panic!("Out of bounds");
        };
        if change.new_max != change.old_max {
            let block = self.get_block(coord);
            block.block().light_updated(self, coord, change.old_max, change.new_max);
        }
        change
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