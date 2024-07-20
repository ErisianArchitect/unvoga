use std::io::{Read, Write};

use bevy::{asset::Assets, prelude::{state_changed, ResMut}, render::mesh::Mesh, utils::tracing::Instrument};

use crate::core::voxel::{blocks::Id, blockstate::BlockState, coord::Coord, direction::Direction, region::timestamp::Timestamp, rendering::voxelmaterial::VoxelMaterial, tag::Tag};

use super::{dirty::Dirty, heightmap::Heightmap, occlusion::Occlusion, query::Query, section::{LightChange, Section, SectionUpdate, StateChange}, update::UpdateRef, MemoryUsage, VoxelWorld, WORLD_BOTTOM, WORLD_HEIGHT};
use crate::core::error::*;

pub struct Chunk {
    pub sections: Box<[Section]>,
    pub heightmap: Heightmap,
    /// The offset block coordinate.
    pub block_offset: Coord,
    pub edit_time: Timestamp,
}

impl Chunk {
    const SECTION_COUNT: usize = WORLD_HEIGHT / 16;
    #[inline(always)]
    pub fn new(offset: Coord) -> Self {
        Self {
            sections: (0..Self::SECTION_COUNT).map(|_| Section::new()).collect(),
            heightmap: Heightmap::new(),
            block_offset: offset,
            edit_time: Timestamp::utc_now(),
        }
    }

    #[inline(always)]
    pub fn mark_modified(&mut self) {
        self.edit_time = Timestamp::utc_now();
    }

    #[inline(always)]
    pub fn dynamic_usage(&self) -> MemoryUsage {
        self.sections.iter().map(|section| section.dynamic_usage()).sum()
    }

    #[inline(always)]
    pub fn get_update_ref(&self, coord: Coord) -> UpdateRef {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_update_ref(coord)
    }

    #[inline(always)]
    pub fn set_update_ref(&mut self, coord: Coord, value: UpdateRef) -> UpdateRef {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        let old = self.sections[section_index].set_update_ref(coord, value);
        if old != value {
            self.mark_modified();
        }
        old
    }

    #[inline(always)]
    pub fn query<'a, T: Query<'a>>(&'a self, coord: Coord) -> T::Output {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].query::<T>(coord)
    }

    #[inline(always)]
    pub fn get_block(&self, coord: Coord) -> Id {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_block(coord)
    }

    #[inline(always)]
    pub fn set_block(&mut self, coord: Coord, value: Id) -> SectionUpdate<StateChange> {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        let nonair = value != Id::AIR;
        self.heightmap.set(Coord::new(coord.x, coord.y - self.block_offset.y, coord.z), nonair);
        let update = self.sections[section_index].set_block(coord, value);
        if update.change.changed() {
            self.mark_modified();
        }
        update
    }

    #[inline(always)]
    pub fn occlusion(&self, coord: Coord) -> Occlusion {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].occlusion(coord)
    }

    #[inline(always)]
    pub fn face_visible(&self, coord: Coord, face: Direction) -> bool {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].face_visible(coord, face)
    }

    #[inline(always)]
    pub fn show_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        // We don't mark the chunk as modified for occlusion updates.
        self.sections[section_index].show_face(coord, face)
    }

    #[inline(always)]
    pub fn hide_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        // We don't mark the chunk as modified for occlusion updates.
        self.sections[section_index].hide_face(coord, face)
    }
    
    /// Returns the maximum of the block light and sky light.
    #[inline(always)]
    pub fn get_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_light(coord)
    }

    #[inline(always)]
    pub fn get_block_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_block_light(coord)
    }

    #[inline(always)]
    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_sky_light(coord)
    }

    #[inline(always)]
    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        let update = self.sections[section_index].set_block_light(coord, level);
        if update.change.changed() {
            self.mark_modified();
        }
        update
    }

    #[inline(always)]
    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        let update = self.sections[section_index].set_sky_light(coord, level);
        if update.change.changed() {
            self.mark_modified();
        }
        update
    }

    #[inline(always)]
    pub fn get_data(&self, coord: Coord) -> Option<&Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_data(coord)
    }

    #[inline(always)]
    pub fn get_data_mut(&mut self, coord: Coord) -> Option<&mut Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_data_mut(coord)
    }

    #[inline(always)]
    pub fn get_or_insert_data(&mut self, coord: Coord, default: Tag) -> &mut Tag {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_or_insert_data(coord, default)
    }

    #[inline(always)]
    pub fn get_or_insert_data_with<T: Into<Tag>, F: FnOnce() -> T>(&mut self, coord: Coord, f: F) -> &mut Tag {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_or_insert_data_with(coord, f)
    }

    #[inline(always)]
    pub fn delete_data(&mut self, coord: Coord) -> Option<Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        let data = self.sections[section_index].delete_data(coord);
        if data.is_some() {
            self.mark_modified();
        }
        data
    }

    #[inline(always)]
    pub fn set_data(&mut self, coord: Coord, tag: Tag) -> Option<Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.mark_modified();
        self.sections[section_index].set_data(coord, tag)
    }

    #[inline(always)]
    pub fn height(&self, x: i32, z: i32) -> i32 {
        self.heightmap.height(x, z) + self.block_offset.y
    }

    #[inline(always)]
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let mut length = 0;
        for i in 0..self.sections.len() {
            // the y offset of the bottom-most block
            length += self.sections[i].write_to(writer)?;
        }
        Ok(length)
    }

    #[inline(always)]
    pub fn read_from<R: Read>(&mut self, reader: &mut R, world: &mut VoxelWorld) -> Result<()> {
        for i in 0..self.sections.len() {
            let y = i as i32 * 16 + self.block_offset.y;
            let offset = Coord::new(self.block_offset.x, y, self.block_offset.z);
            let marked = self.sections[i].read_from(reader, world, offset)?;
            // if offset is not in the render bounds, we don't want to add it
            // to the dirty_queue
            if world.render_bounds().contains(offset) {
                world.dirty_sections.push(offset.section_coord());
            }
        }
        Ok(())
    }

    #[inline(always)]
    pub fn unload(&mut self, world: &mut VoxelWorld) {
        for i in 0..self.sections.len() {
            let y = i as i32 * 16 + self.block_offset.y;
            let offset = Coord::new(self.block_offset.x, y, self.block_offset.z);
            let marked = self.sections[i].unload(world);
            if world.render_bounds().contains(offset) {
                world.dirty_sections.push(offset.section_coord());
            }
        }
    }

}

#[cfg(test)]
mod testing_sandbox {
    use super::*;
    #[test]
    fn sandbox() {
        let neg8 = -8;
        let neg8 = neg8 as usize;
        let bit_count = (31 + 8 - 1) & neg8;
        println!("{bit_count}");
    }
}

#[cfg(test)]
mod tests {
    use crate::core::voxel::{direction::Direction, world::chunk::Occlusion};

    #[test]
    fn occlusion_test() {
        let mut occlusion = Occlusion::UNOCCLUDED;
        Direction::iter().for_each(|dir| {
            assert!(!occlusion.hidden(dir));
            assert!(occlusion.visible(dir));
            occlusion.hide(dir);
            assert!(occlusion.hidden(dir));
            assert!(!occlusion.visible(dir));
            occlusion.show(dir);
            assert!(!occlusion.hidden(dir));
            assert!(occlusion.visible(dir));
        });
        assert_eq!(occlusion, Occlusion::UNOCCLUDED);
        assert!(Occlusion::OCCLUDED.fully_occluded());
    }
    
}
