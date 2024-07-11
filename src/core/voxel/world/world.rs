
use std::collections::VecDeque;

use rollgrid::{RollGrid2D, RollGrid3D};

use crate::core::voxel::{blocks::StateRef, coord::Coord, engine::VoxelEngine};

use super::chunk::{Chunk, LightChange, Section, StateChange};

// Make sure this value is always a multiple of 16 and
// preferably a multiple of 128.
pub const WORLD_HEIGHT: usize = 640;
pub const WORLD_BOTTOM: i32 = -400;
pub const WORLD_TOP: i32 = WORLD_BOTTOM + WORLD_HEIGHT as i32;
pub const WORLD_SIZE_MAX: usize = 20;
/// The pad size is the added chunk width for light updates.
/// If WORLD_SIZE_MAX is the range that is visible to the player,
/// PADDED_WORLD_SIZE_MAX is the range that  has light updates (since
/// light updates can span multple chunks).
pub const WORLD_SIZE_MAX_PAD: usize = 2;
pub const PADDED_WORLD_SIZE_MAX: usize = WORLD_SIZE_MAX + WORLD_SIZE_MAX_PAD;

pub struct World {
    chunks: RollGrid2D<Chunk>,
    render_chunks: RollGrid3D<()>,
    dirty_sections: Vec<Coord>,
}

impl World {
    pub fn new(render_distance: usize, center: Coord) -> Self {
        if render_distance > PADDED_WORLD_SIZE_MAX {
            panic!("Size greater than {PADDED_WORLD_SIZE_MAX} (PADDED_WORLD_SIZE_MAX)");
        }
        let full_size = render_distance * 2;
        let size = render_distance as i32;
        let (x, y, z) = (center.x, center.y, center.z);
        let (offx, offy, offz) = (x - 8, y - 8, z - 8);
        let size_offset = (size - 1) * 16;
        let (newx, newy, newz) = (
            (offx & !0xF) - size_offset,
            (offy & !0xF) - size_offset,
            (offz & !0xF) - size_offset
        );
        Self {
            chunks: RollGrid2D::new_with_init(full_size, full_size, (newx / 16, newz / 16), |(x, z): (i32, i32)| {
                Some(Chunk::new(Coord::new(x * 16, WORLD_BOTTOM, z * 16)))
            }),
            render_chunks: RollGrid3D::new_with_init(full_size, full_size, full_size, (newx / 16, newy / 16, newz / 16), |pos: Coord| {
                Some(())
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
            return StateRef::AIR;
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
            return LightChange::default();
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
            return LightChange::default();
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

    pub fn get_height(&self, x: i32, z: i32) -> Option<i32> {
        let chunk_x = x / 16;
        let chunk_z = z / 16;
        if let Some(chunk) = self.chunks.get((x, z)) {
            Some(chunk.height(x, z))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{blockstate, core::voxel::{block::Block, blocks::{self, StateRef}, coord::Coord, world::world::WORLD_TOP}};

    use super::World;

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

    #[test]
    fn world_test() {
        println!("World Test");
        let mut world = World::new(16, (0, 0));
        struct DirtBlock;
        impl Block for DirtBlock {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn name(&self) -> &str {
                "dirt"
            }

            fn default_state(&self) -> crate::core::voxel::blockstate::BlockState {
                blockstate!(dirt)
            }
        }
        blocks::register_block(DirtBlock);
        let dirt = blocks::register_state(blockstate!(dirt));

        let now = std::time::Instant::now();
        for y in 0..64 {
            for z in 0..64 {
                for x in 0..64 {
                    let coord = Coord::new(x, y, z);
                    world.set_block(coord, dirt);
                    world.set_block_light(coord, 7);
                    world.set_sky_light(coord, 15);
                }
            }
        }
        let elapsed = now.elapsed();
        println!("Elapsed: {}", elapsed.as_secs_f64());
        for coord in world.dirty_sections.iter() {
            println!("Dirty: {coord}");
        }
        println!("Dirty len: {}", world.dirty_sections.len());
        return;
        let block = world.get_block(Coord::new(143,WORLD_TOP-1,0));
        println!("{}", block);
        world.set_block(Coord::new(143,WORLD_TOP-1,0), dirt);
        let block = world.get_block(Coord::new(143, WORLD_TOP-1, 0));
        println!("{}", block);
        let coord = Coord::new(143, WORLD_TOP-1, 0);
        let light = world.get_sky_light(coord);
        println!("Light: {light}");
        world.set_sky_light(coord, 13);
        let light = world.get_sky_light(coord);
        println!("Light: {light}");
        // for offset in &world.dirty_sections {
        //     println!("{offset}");
        // }
        for i in 0..32 {
            let coord = Coord::new(i, 0, 0);
            world.set_block(coord, dirt);
            world.set_block_light(coord, 15);
            world.set_sky_light(coord, 7);
        }
        for i in 0..32 {
            let coord = Coord::new(i, 0, 0);
            let block = world.set_block(coord, StateRef::AIR);
            let block_light = world.set_block_light(coord, 0).old_level;
            let sky_light = world.set_sky_light(coord, 0).old_level;
            println!("{block} {block_light} {sky_light}")
        }
        let sect = Coord::splat(0).section_coord();
        if let Some(sect) = world.get_section(sect) {
            println!("     Blocks is None: {}", sect.blocks.is_none());
            println!("Block Light is None: {}", sect.block_light.is_none());
            println!("  Sky Light is None: {}", sect.sky_light.is_none());
        }
        for offset in &world.dirty_sections {
            println!("{offset}");
        }
        let now = std::time::Instant::now();
        let mut timer = std::time::Instant::now();
    }
}
