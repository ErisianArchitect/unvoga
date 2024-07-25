#![allow(unused)]
use bevy::{asset::Assets, prelude::{state_changed, ResMut}, render::mesh::Mesh, utils::tracing::Instrument};

use crate::{core::{collections::objectpool::PoolId, error::*}, prelude::{SwapVal, Writeable}};
use crate::core::voxel::{blocks::Id, blockstate::BlockState, coord::Coord, direction::Direction, rendering::voxelmaterial::VoxelMaterial, tag::Tag};

use super::{blockdata::{BlockDataContainer, BlockDataRef}, dirty::Dirty, heightmap::Heightmap, io::{read_block_data, read_enabled, read_section_blocks, read_section_light, read_section_occlusions}, occlusion::Occlusion, query::Query, update::UpdateRef, DirtyIdMarker, MemoryUsage, SaveIdMarker, VoxelWorld, WORLD_HEIGHT};
use crate::core::io::*;

// 4096*4+4096+2048+2048+4096*2
// 32768 bytes
/// A single 16x16x16 (4096) block section.
/// This includes:
///     blocks
///     occlusion flags
///     sky light
///     block light
///     block data
pub struct Section {
    pub blocks: Option<Box<[Id]>>,
    pub occlusion: Option<Box<[Occlusion]>>,
    pub block_light: Option<Box<[u8]>>,
    pub sky_light: Option<Box<[u8]>>,
    pub block_data_refs: Option<Box<[BlockDataRef]>>,
    pub update_refs: Option<Box<[UpdateRef]>>,
    pub block_data: BlockDataContainer,
    pub block_count: u16,
    pub occlusion_count: u16,
    pub block_light_count: u16,
    pub sky_light_count: u16,
    pub block_data_count: u16,
    pub update_ref_count: u16,
    pub blocks_dirty: Dirty,
    pub light_dirty: Dirty,
    /// I use this flag to determine if this section has been added
    /// to the dirty queue in the [VoxelWorld].
    pub section_dirty: Dirty,
    pub dirty_id: PoolId<DirtyIdMarker>,
}

impl Section {
    
    pub fn new() -> Self {
        Self {
            blocks: None,
            occlusion: None,
            block_light: None,
            sky_light: None,
            block_data_refs: None,
            update_refs: None,
            block_data: BlockDataContainer::new(),
            block_count: 0,
            occlusion_count: 0,
            block_light_count: 0,
            sky_light_count: 0,
            block_data_count: 0,
            update_ref_count: 0,
            blocks_dirty: Dirty::new(),
            light_dirty: Dirty::new(),
            section_dirty: Dirty::new(),
            dirty_id: PoolId::NULL,
        }
    }

    /// Gets the dynamic memory usage. (This doesn't include the usage for [Tag]s; TODO)
    pub fn dynamic_usage(&self) -> MemoryUsage {
        let mut usage = MemoryUsage::new(0, 0);
        // let mut printed = false;
        if self.blocks.is_some() {
            // println!("################");
            // printed = true;
            // println!("Blocks Used");
            usage.used += 4096 * std::mem::size_of::<Id>();
        }
        usage.total += 4096 * std::mem::size_of::<Id>();
        if self.occlusion.is_some() {
            // if !printed {
            //     println!("################");
            //     printed = true;
            // }
            // println!("Occlusion Used");
            // println!("{:?}", &self.occlusion);
            usage.used += 4096 * std::mem::size_of::<Occlusion>();
        }
        usage.total += 4096 * std::mem::size_of::<Occlusion>();
        if self.block_light.is_some() {
            // if !printed {
            //     println!("################");
            //     printed = true;
            // }
            // println!("Block Light Used");
            usage.used += 2048;
        }
        usage.total += 2048;
        if self.sky_light.is_some() {
            // if !printed {
            //     println!("################");
            //     printed = true;
            // }
            // println!("Sky Light Used");
            usage.used += 2048;
        }
        usage.total += 2048;
        if self.block_data_refs.is_some() {
            // if !printed {
            //     println!("################");
            //     printed = true;
            // }
            // println!("Block Data Used");
            usage.used += 4096*std::mem::size_of::<BlockDataRef>();
        }
        usage.total += 4096*std::mem::size_of::<BlockDataRef>();
        usage + self.block_data.dynamic_usage()
    }

    /// Gets the index in the 16x16x16 [Section].
    /// This is yzx order (x | z << 4 | y << 8)
    pub fn index(coord: Coord) -> usize {
        let x = (coord.x & 0xF) as usize;
        let y = (coord.y & 0xF) as usize;
        let z = (coord.z & 0xF) as usize;
        x | z << 4 | y << 8
    }

    
    pub fn coord(index: u16) -> Coord {
        let x = index & 0xf;
        let y = index >> 8 & 0xf;
        let z = index >> 4 & 0xf;
        Coord::new(x as i32, y as i32, z as i32)
    }

    
    pub fn query<'a, T: Query<'a>>(&'a self, coord: Coord) -> T::Output {
        let index = Section::index(coord);
        T::read(self, index)
    }

    
    pub fn get_update_ref(&self, coord: Coord) -> UpdateRef {
        if let Some(refs) = &self.update_refs {
            let index = Section::index(coord);
            refs[index]
        } else {
            UpdateRef::NULL
        }
    }

    
    pub fn set_update_ref(&mut self, coord: Coord, value: UpdateRef) -> UpdateRef {
        if self.update_refs.is_none() {
            if value.null() {
                return UpdateRef::NULL;
            } else {
                self.update_refs = Some(make_empty_update_refs());
            }
        }
        let refs = self.update_refs.as_mut().unwrap();
        let index = Section::index(coord);
        let mut old = value;
        std::mem::swap(&mut old, &mut refs[index]);
        if old != value {
            // Decrement
            if old == UpdateRef::NULL {
                self.update_ref_count += 1;
            } else {
                self.update_ref_count -= 1;
                if self.update_ref_count == 0 {
                    self.update_refs = None;
                }
            }
        }
        old
    }

    /// Get the [Id] at the `coord`.
    pub fn get_block(&self, coord: Coord) -> Id {
        if let Some(blocks) = &self.blocks {
            let index = Section::index(coord);
            blocks[index]
        } else {
            // If self.blocks is None, that means it's all air.
            Id::AIR
        }
    }

    /// Set the [Id] at the `coord`.
    pub fn set_block(&mut self, coord: Coord, state: Id) -> SectionUpdate<StateChange> {
        if self.blocks.is_none() {
            // if state isn't air and blocks is None, create an empty block array.
            if !state.is_air() {
                self.blocks = Some(make_empty_section_blocks());
            } else {
                // state was air, and blocks was None (all air), so the state is unchanged.
                return SectionUpdate::new(StateChange::Unchanged);
            }
        }
        let index = Section::index(coord);
        let blocks = self.blocks.as_mut().unwrap();
        let mut old = state;
        std::mem::swap(&mut blocks[index], &mut old);
        // Check that the new state is different than the old state
        if state != old {
            if old.is_air() && !state.is_air() {
                // if the old state is air and the new state isn't air, increment the block_count.
                // this allows to keep track of when the section is all air blocks.
                // when the block_count is 0, it's all air.
                // we're effectively counting the non-air blocks here.
                self.block_count += 1;
            } else if state.is_air() && !old.is_air() {
                // if the new state is air and the old state isn't air,
                // decrement the block_count then check if the block_count has reached zero.
                // If the block count is zero, set blocks to None to free up the memory.
                self.block_count -= 1;
                if self.block_count == 0 {
                    self.blocks = None;
                }
            }
            // mark the blocks as dirty so that the engine knows to rebuild the mesh.
            self.blocks_dirty.mark();
            if self.section_dirty.mark() {
                SectionUpdate::new_dirty(StateChange::Changed(old))
            } else {
                SectionUpdate::new(StateChange::Changed(old))
            }
        } else {
            SectionUpdate::new(StateChange::Unchanged)
        }
    }

    /// Gets the [Occlusion] flags for the block.
    /// These are the faces that are hidden.
    pub fn occlusion(&self, coord: Coord) -> Occlusion {
        if let Some(occlusion) = &self.occlusion {
            let index = Section::index(coord);
            occlusion[index]
        } else {
            Occlusion::UNOCCLUDED
        }
    }

    /// Checks if a face is visible.
    pub fn face_visible(&self, coord: Coord, face: Direction) -> bool {
        if let Some(occlusion) = &self.occlusion {
            let index = Section::index(coord);
            occlusion[index].visible(face)
        } else {
            // Faces are visible by default.
            true
        }
    }

    /// Show a face.
    pub fn show_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        if self.occlusion.is_none() {
            return SectionUpdate::new(true);
        }
        let Some(occlusion) = &mut self.occlusion else {
            unreachable!()
        };
        let index = Section::index(coord);
        let old_flags = occlusion[index];
        let old = occlusion[index].show(face);
        let new_flags = occlusion[index];
        if new_flags == Occlusion::UNOCCLUDED && old_flags != Occlusion::UNOCCLUDED {
            // decrement the occlusion_count which keeps track of how many blocks are occluded.
            // when the occlusion_count is 0, self.occlusion is set to None to free up memory.
            self.occlusion_count -= 1;
            if self.occlusion_count == 0 {
                self.occlusion = None;
            }
        }
        if !old {
            self.blocks_dirty.mark();
            if self.section_dirty.mark() {
                return SectionUpdate::new_dirty(old);
            }
        }
        SectionUpdate::new(old)
    }

    /// Hide a face.
    pub fn hide_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        let occlusion = self.occlusion.get_or_insert_with(make_empty_occlusion_data);
        let index = Section::index(coord);
        let old_flags = occlusion[index];
        let old = occlusion[index].hide(face);
        if old_flags == Occlusion::UNOCCLUDED {
            // increment the occlusion_count to keep track of how many blocks are occluded.
            // this allows for being able to free up memory when no blocks are occluded.
            self.occlusion_count += 1;
        }
        if old {
            self.blocks_dirty.mark();
            if self.section_dirty.mark() {
                return SectionUpdate::new_dirty(old);
            }
        }
        SectionUpdate::new(old)
    }

    /// Returns the maximum of the block light and sky light.
    pub fn get_light(&self, coord: Coord) -> u8 {
        let block_light = self.get_block_light(coord);
        let sky_light = self.get_sky_light(coord);
        block_light.max(sky_light)
    }

    /// Gets the block light. The block light is the light that comes
    /// from other blocks.
    pub fn get_block_light(&self, coord: Coord) -> u8 {
        if let Some(block_light) = &self.block_light {
            let index = Section::index(coord);
            // The block light is stored in an array of 2048 bytes, although it's 4096 values.
            // That's 2 values per byte.
            // Using some simple math, we can unpack the value.
            let mask_index = index / 2;
            let sub_index = (index & 1) * 4;
            block_light[mask_index] >> sub_index & 0xF
        } else {
            0
        }
    }

    /// Sets the block light. The block light is the light that comes
    /// from other blocks.
    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let level = level.min(15);
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1);
        let shift = sub_index * 4;
        // Get the sky light to compare for the max/old max
        let sky_light = if let Some(sky_light) = &self.sky_light {
            sky_light[mask_index] >> shift & 0xF
        } else {
            0
        };
        if self.block_light.is_none() {
            if level != 0 {
                // if level isn't 0, we want to make an empty lightmap
                self.block_light = Some(make_empty_section_light());
            } else {
                // No change has occurred
                return SectionUpdate::new(LightChange {
                    old_max: sky_light,
                    old_level: 0,
                    new_level: 0,
                    new_max: sky_light,
                });
            }
        }
        let block_light = self.block_light.as_mut().unwrap();
        let other_index = ((sub_index as i32 - 1) & 1) as usize;
        let other_shift = other_index * 4;
        let old_level = block_light[mask_index] >> shift & 0xF;
        let old_max = sky_light.max(old_level);
        let change = LightChange {
            old_max,
            old_level,
            new_level: level,
            new_max: sky_light.max(level),
        };
        if level == old_level {
            return SectionUpdate::new(change);
        }
        let other = block_light[mask_index] & (0xF << other_shift);
        block_light[mask_index] = other | (level << shift);

        // update the block_light_count so that memory can be freed when it reaches 0.
        if level != 0 && old_level == 0 {
            self.block_light_count += 1;
        } else if level == 0 && old_level != 0 {
            self.block_light_count -= 1;
            if self.block_light_count == 0 {
                self.block_light = None;
            }
        }
        if change.new_max != change.old_max {
            self.light_dirty.mark();
            if self.section_dirty.mark() {
                SectionUpdate::new_dirty(change)
            } else {
                SectionUpdate::new(change)
            }
        } else {
            SectionUpdate::new(change)
        }
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        if let Some(sky_light) = &self.sky_light {
            let index = Section::index(coord);
            let mask_index = index / 2;
            let sub_index = (index & 1) * 4;
            sky_light[mask_index] >> sub_index & 0xF
        } else {
            15
        }
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let level = level.min(15);
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1);
        let shift = sub_index * 4;
        // Get the block light to compare for the max/old max
        let block_light = if let Some(block_light) = &self.block_light {
            block_light[mask_index] >> shift & 0xF
        } else {
            15
        };
        if self.sky_light.is_none() {
            if level != 15 {
                self.sky_light = Some(make_empty_section_light());
            } else {
                return SectionUpdate::new(LightChange {
                    old_max: block_light,
                    old_level: 15,
                    new_level: 15,
                    new_max: block_light,
                });
            }
        }
        let sky_light = self.sky_light.as_mut().unwrap();
        // light values are packed as 4-bit nibbles
        // gotta unpack the other value
        let other_index = ((sub_index as i32 - 1) & 1) as usize;
        let other_shift = other_index * 4;
        let old_level = sky_light[mask_index] >> shift & 0xF;
        let old_max = block_light.max(old_level);
        let change = LightChange {
            old_max,
            old_level,
            new_level: level,
            new_max: block_light.max(level),
        };
        if level == old_level {
            return SectionUpdate::new(change);
        }
        let other = sky_light[mask_index] & (0xF << other_shift);
        sky_light[mask_index] = other | (level << shift);
        if level != 15 && old_level == 15 {
            self.sky_light_count += 1;
        } else if level == 15 && old_level != 15 {
            self.sky_light_count -= 1;
            if self.sky_light_count == 0 {
                self.sky_light = None;
            }
        }
        if change.new_max != change.old_max {
            self.light_dirty.mark();
            if self.section_dirty.mark() {
                SectionUpdate::new_dirty(change)
            } else {
                SectionUpdate::new(change)
            }
        } else {
            SectionUpdate::new(change)
        }
    }

    pub fn get_data(&self, coord: Coord) -> Option<&Tag> {
        let Some(data) = &self.block_data_refs else {
            return None;
        };
        let index = Section::index(coord);
        let dref = data[index];
        self.block_data.get(dref)
    }

    pub fn get_data_mut(&mut self, coord: Coord) -> Option<&mut Tag> {
        let Some(data) = &mut self.block_data_refs else {
            return None;
        };
        let index = Section::index(coord);
        let dref = data[index];
        self.block_data.get_mut(dref)
    }

    pub fn get_or_insert_data(&mut self, coord: Coord, default: Tag) -> &mut Tag {
        self.get_or_insert_data_with(coord, || default)
    }

    pub fn get_or_insert_data_with<T: Into<Tag>, F: FnOnce() -> T>(&mut self, coord: Coord, f: F) -> &mut Tag {
        let data = self.block_data_refs.get_or_insert_with(make_empty_block_data);
        let index = Section::index(coord);
        let dref = data[index];
        if dref.null() {
            let insert: Tag = f().into();
            let newref = self.block_data.insert(insert);
            data[index] = newref;
            self.block_data.get_mut(newref).unwrap()
        } else {
            self.block_data.get_mut(dref).unwrap()
        }
    }

    pub fn delete_data(&mut self, coord: Coord) -> Option<Tag> {
        let Some(data) = &mut self.block_data_refs else {
            return None;
        };
        let index = Section::index(coord);
        let mut old = BlockDataRef::NULL;
        std::mem::swap(&mut old, &mut data[index]);
        if !old.null() {
            self.block_data_count -= 1;
            if self.block_data_count == 0 {
                self.block_data_refs = None;
            }
            Some(self.block_data.delete(old))
        } else {
            None
        }
    }

    pub fn set_data(&mut self, coord: Coord, tag: Tag) -> Option<Tag> {
        let index = Section::index(coord);
        let (oldref, data) = if let Some(data) = &mut self.block_data_refs {
            (data[index], data)
        } else {
            let data = self.block_data_refs.insert((0..4096).map(|_| BlockDataRef::NULL).collect());
            (BlockDataRef::NULL, data)
        };
        let old = if oldref.null() {
            self.block_data_count += 1;
            None
        } else {
            Some(self.block_data.delete(oldref))
        };
        let new = self.block_data.insert(tag);
        data[index] = new;
        old
    }

    /// Copy max block/sky light into dest (where dest is 4096 slot lightmap stored yzx order).
    pub fn copy_lightmap(&self, dest: &mut [u8]) {
        (0..4096).for_each(|i| {
            let mask_index = i / 2;
            let sub_index = (i & 1);
            let shift = sub_index * 4;
            let block_light = if let Some(block_light) = &self.block_light {
                block_light[mask_index] >> shift & 0xF
            } else {
                0
            };
            let sky_light = if let Some(sky_light) = &self.sky_light {
                sky_light[mask_index] >> shift & 0xF
            } else {
                0
            };
            let light = block_light.max(sky_light);
            dest[i] = light;
        });
    }

    // Serialization/Deserialization
    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<u64> {
        // First write the flags that determine what data this section has.
        // If the section is empty, this will write a 0 byte and return.
        use super::io::*;
        if self.block_count == 0
        && self.occlusion_count == 0
        && self.block_light_count == 0
        && self.sky_light_count == 0
        && self.block_data_count == 0
        && self.update_ref_count == 0 {
            return false.write_to(writer);
        }
        let mut length = true.write_to(writer)?;
        length += write_section_blocks(writer, &self.blocks)?;
        length += write_section_occlusions(writer, &self.occlusion)?;
        length += write_section_light(writer, &self.block_light)?;
        length += write_section_light(writer, &self.sky_light)?;
        length += write_block_data(writer, &self.block_data_refs, &self.block_data, self.block_data_count)?;
        length += write_enabled(writer, &self.update_refs)?;
        Ok(length)
    }

    pub fn read_from<R: std::io::Read>(&mut self, reader: &mut R, world: &mut VoxelWorld, offset: Coord) -> Result<bool> {
        let flag = bool::read_from(reader)?;
        if !flag {
            return Ok(self.unload(world));
        }
        read_section_blocks(reader, &mut self.blocks, &mut self.block_count)?;
        read_section_occlusions(reader, &mut self.occlusion, &mut self.occlusion_count)?;
        read_section_light(reader, &mut self.block_light, &mut self.block_light_count)?;
        read_section_light(reader, &mut self.sky_light, &mut self.sky_light_count)?;
        read_block_data(reader, &mut self.block_data_refs, &mut self.block_data, &mut self.block_data_count)?;
        // We're assuming that the old data hasn't been safely unloaded yet.
        let update_refs = self.update_refs.get_or_insert_with(|| (0..4096).map(|_| UpdateRef::NULL).collect());
        update_refs.iter_mut().for_each(|uref| {
            if !uref.null() {
                world.update_queue.remove(*uref);
                *uref = UpdateRef::NULL;
            }
        });
        read_enabled(reader, |index| {
            let block_coord = Section::coord(index) + offset;
            // You can't use world.set_enabled here because world.set_enabled needs access to chunks, which is already being borrowed.
            let uref = world.update_queue.push(block_coord);
            update_refs[index as usize] = uref;
        }, &mut self.update_ref_count)?;
        self.blocks_dirty.mark();
        self.light_dirty.mark();
        return Ok(self.section_dirty.mark());
    }

    fn disable_all(&mut self, world: &mut VoxelWorld) {
        if let Some(refs) = self.update_refs.take() {
            assert!(!world.lock_update_queue, "Update queue was locked.");
            refs.into_iter().for_each(|&uref| {
                world.update_queue.remove(uref);
            });
        }
        self.update_ref_count = 0;
    }

    pub fn unload(&mut self, world: &mut VoxelWorld) -> bool {
        let dirty_id = self.dirty_id.swap(PoolId::NULL);
        if !dirty_id.null() {
            world.dirty_queue.remove(dirty_id);
        }
        self.blocks = None;
        self.occlusion = None;
        self.block_light = None;
        self.sky_light = None;
        self.block_data_refs = None;
        self.disable_all(world);
        self.block_data.clear();
        self.block_count = 0;
        self.occlusion_count = 0;
        self.block_light_count = 0;
        self.sky_light_count = 0;
        self.block_data_count = 0;
        self.blocks_dirty.mark();
        self.light_dirty.mark();
        self.section_dirty.mark()
    }

    pub fn is_empty(&self) -> bool {
        self.block_count == 0
        && self.block_light_count == 0
        && self.sky_light_count == 0
    }
}



#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LightChange {
    pub old_max: u8,
    pub old_level: u8,
    pub new_max: u8,
    pub new_level: u8,
}

impl LightChange {
    pub fn changed(self) -> bool {
        self.new_level != self.old_level
    }

    pub fn max_changed(self) -> bool {
        self.new_max != self.old_max
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateChange {
    Unchanged,
    Changed(Id)
}

impl StateChange {
    pub fn changed(self) -> bool {
        matches!(self, StateChange::Changed(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionUpdate<T: Clone + Copy + PartialEq + Eq + std::hash::Hash> {
    /// Determines if the section was marked dirty during this update.
    pub marked_dirty: bool,
    pub change: T,
}

impl<T: Clone + Copy + PartialEq + Eq + std::hash::Hash> SectionUpdate<T> {
    pub fn new_dirty(change: T) -> Self {
        Self {
            marked_dirty: true,
            change
        }
    }

    /// Not marked dirty
    pub fn new(change: T) -> Self {
        Self {
            marked_dirty: false,
            change
        }
    }
}

/// Create empty [Section] blocks.
fn make_empty_section_blocks() -> Box<[Id]> {
    (0..4096).map(|_| Id::AIR).collect()
}

/// Create empty [Section] lightmap.
fn make_empty_section_light() -> Box<[u8]> {
    (0..2048).map(|_| 15).collect()
}

/// Create empty [Section] block data ref grid.
fn make_empty_block_data() -> Box<[BlockDataRef]> {
    (0..4096).map(|_| BlockDataRef::NULL).collect()
}

fn make_empty_occlusion_data() -> Box<[Occlusion]> {
    (0..4096).map(|_| Occlusion::UNOCCLUDED).collect()
}

fn make_empty_update_refs() -> Box<[UpdateRef]> {
    (0..4096).map(|_| UpdateRef::NULL).collect()
}