use bevy::{asset::Assets, prelude::{state_changed, ResMut}, render::mesh::Mesh, utils::tracing::Instrument};

use crate::core::voxel::{blocks::StateRef, blockstate::BlockState, coord::Coord, direction::Direction, rendering::voxelmaterial::VoxelMaterial, tag::Tag};

use super::{dirty::Dirty, heightmap::Heightmap, occlusion::Occlusion, section::{LightChange, Section, SectionUpdate, StateChange}, WORLD_HEIGHT};

pub struct Chunk {
    pub sections: Box<[Section]>,
    pub heightmap: Heightmap,
    /// The offset block coordinate.
    pub block_offset: Coord,
}

impl Chunk {
    const SECTION_COUNT: usize = WORLD_HEIGHT / 16;
    pub fn new(offset: Coord) -> Self {
        Self {
            sections: (0..Self::SECTION_COUNT).map(|_| Section::new()).collect(),
            heightmap: Heightmap::new(),
            block_offset: offset,
        }
    }

    pub fn dynamic_usage(&self) -> usize {
        self.sections.iter().map(|section| section.dynamic_usage()).sum()
    }

    pub fn get(&self, coord: Coord) -> StateRef {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get(coord)
    }

    pub fn set(&mut self, coord: Coord, value: StateRef) -> SectionUpdate<StateChange> {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        let nonair = value != StateRef::AIR;
        self.heightmap.set(Coord::new(coord.x, coord.y - self.block_offset.y, coord.z), nonair);
        self.sections[section_index].set(coord, value)
    }

    pub fn occlusion(&self, coord: Coord) -> Occlusion {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].occlusion(coord)
    }

    pub fn face_visible(&self, coord: Coord, face: Direction) -> bool {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].face_visible(coord, face)
    }

    pub fn show_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].show_face(coord, face)
    }

    pub fn hide_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].hide_face(coord, face)
    }
    
    /// Returns the maximum of the block light and sky light.
    pub fn get_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_light(coord)
    }

    pub fn get_block_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_block_light(coord)
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_sky_light(coord)
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].set_block_light(coord, level)
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].set_sky_light(coord, level)
    }

    pub fn get_data(&self, coord: Coord) -> Option<&Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_data(coord)
    }

    pub fn get_data_mut(&mut self, coord: Coord) -> Option<&mut Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_data_mut(coord)
    }

    pub fn get_or_insert_data(&mut self, coord: Coord, default: Tag) -> &mut Tag {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_or_insert_data(coord, default)
    }

    pub fn get_or_insert_data_with<T: Into<Tag>, F: FnOnce() -> T>(&mut self, coord: Coord, f: F) -> &mut Tag {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_or_insert_data_with(coord, f)
    }

    pub fn delete_data(&mut self, coord: Coord) -> Option<Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].delete_data(coord)
    }

    pub fn set_data(&mut self, coord: Coord, tag: Tag) -> Option<Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].set_data(coord, tag)
    }

    pub fn height(&self, x: i32, z: i32) -> i32 {
        self.heightmap.height(x, z) + self.block_offset.y
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
