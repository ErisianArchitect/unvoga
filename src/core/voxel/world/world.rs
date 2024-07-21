use std::path::{Path, PathBuf};
use std::{collections::VecDeque, iter::Sum};

use bevy::{asset::Handle, render::mesh::Mesh};
use hashbrown::HashMap;
use super::chunkcoord::ChunkCoord;
use super::occlusion::Occlusion;
use super::query::Query;
use rollgrid::{rollgrid2d::*, rollgrid3d::*};
use super::section::{LightChange, Section, StateChange};
use super::update::{BlockUpdateQueue, UpdateRef};

use crate::core::collections::objectpool::{ObjectPool, PoolId};
use crate::core::math::grid::{calculate_region_min, calculate_region_requirement};
use crate::core::voxel::region::regionfile::RegionFile;
use crate::core::voxel::region::timestamp::Timestamp;
use crate::core::{math::grid::calculate_center_offset, voxel::{blocks::Id, coord::Coord, direction::Direction, engine::VoxelEngine, faces::Faces, rendering::voxelmaterial::VoxelMaterial}};
use crate::prelude::SwapVal;

use super::chunk::Chunk;

use crate::core::voxel::tag::Tag;
use crate::core::error::*;

// Make sure this value is always a multiple of 16 and
// preferably a multiple of 128.
pub const WORLD_HEIGHT: usize = 640;
pub const WORLD_TOP: i32 = WORLD_BOTTOM + WORLD_HEIGHT as i32;
pub const WORLD_BOTTOM: i32 = -400;
pub const WORLD_SIZE_MAX: usize = 64;
/// The pad size is the added chunk width for light updates.
/// If WORLD_SIZE_MAX is the range that is visible to the player,
/// PADDED_WORLD_SIZE_MAX is the range that  has light updates (since
/// light updates can span multple chunks).
pub const WORLD_SIZE_PAD: usize = 2;
pub const PADDED_WORLD_SIZE_MAX: usize = WORLD_SIZE_MAX + WORLD_SIZE_PAD;

macro_rules! cast_coord {
    ($name:ident) => {
        let $name: (i32, i32, i32) = $name.into();
        let $name: Coord = $name.into();
    };
}

/* todo
World Edit
*/

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirtyIdMarker;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SaveIdMarker;

pub struct VoxelWorld {
    /// Determines if the render world has been initialized.
    pub initialized: bool,
    pub dirty_sections: Vec<Coord>,
    pub dirty_queue: ObjectPool<Coord, DirtyIdMarker>,
    pub save_queue: ObjectPool<ChunkCoord, SaveIdMarker>,
    pub chunks: RollGrid2D<Chunk>,
    pub render_chunks: RollGrid3D<RenderChunk>,
    // I gotta figure out grid positioning for the loaded region files.
    // I can give it a buffer of 1 region so that there's room for the
    // world to move around without overflow.
    // I have an idea that involes rounding up to the next multiple of 32.
    // (n + 30) & -32
    // You'll have to do some hacky stuff to make -32u32.
    pub regions: RollGrid2D<RegionFile>,
    pub update_queue: BlockUpdateQueue,
    pub lock_update_queue: bool,
    /// (Coord, new)
    pub update_modification_queue: Vec<(Coord, bool)>,
    /// The value is the index in the update_modification_queue where
    /// the modification is stored.
    pub update_modification_map: HashMap<Coord, u32>,
    pub world_directory: PathBuf,
    pub subworld_directory: PathBuf,
    render_distance: i32,
}

impl VoxelWorld {
    /// The maximum bounds of the world.
    const WORLD_BOUNDS: Bounds3D = Bounds3D {
        min: (i32::MIN, WORLD_BOTTOM, i32::MIN),
        max: (i32::MAX, WORLD_TOP, i32::MAX)
    };
    /// Create a new world centered at the specified block coordinate with the (chunk) render distance specified.
    /// The resulting width in chunks will be `render_distance * 2`.
    pub fn new<P: AsRef<Path>>(render_distance: u8, center: Coord, directory: P) -> Self {
        let mut center = center;
        // clamp Y to world Y range
        // center.y = center.y.min(WORLD_TOP).max(WORLD_BOTTOM);
        if render_distance as usize + WORLD_SIZE_PAD > PADDED_WORLD_SIZE_MAX {
            panic!("Size greater than {PADDED_WORLD_SIZE_MAX} (PADDED_WORLD_SIZE_MAX)");
        }
        let pad_distance = (render_distance as usize + WORLD_SIZE_PAD);
        let pad_size = pad_distance * 2;
        let render_size = render_distance as usize * 2;
        let render_height = render_size.min(WORLD_HEIGHT);
        let (chunk_x, chunk_z) = calculate_center_offset(pad_distance as i32, center, Some(Self::WORLD_BOUNDS)).chunk_coord().xz();
        let region_size = calculate_region_requirement(pad_size as i32);
        let region_min = calculate_region_min((chunk_x, chunk_z));
        let (render_x, render_y, render_z) = calculate_center_offset(render_distance as i32, center, Some(Self::WORLD_BOUNDS)).section_coord().xyz();
        let directory = directory.as_ref();
        std::fs::create_dir_all(directory);
        let subworlds = directory.join("subworlds");
        std::fs::create_dir(&subworlds);
        let main_world = subworlds.join("main");
        std::fs::create_dir(&main_world);
        Self {
            initialized: false,
            subworld_directory: main_world,
            dirty_queue: ObjectPool::new(),
            save_queue: ObjectPool::new(),
            render_distance: render_distance as i32,
            world_directory: directory.to_owned(),
            dirty_sections: Vec::new(),
            chunks: RollGrid2D::new_with_init(pad_size, pad_size, (chunk_x, chunk_z), |(x, z): (i32, i32)| {
                Some(Chunk::new(Coord::new(x * 16, WORLD_BOTTOM, z * 16)))
            }),
            render_chunks: RollGrid3D::new_with_init(render_size, render_size.min(WORLD_HEIGHT / 16), render_size, (render_x, render_y, render_z), |pos: Coord| {
                Some(RenderChunk {
                    mesh: Handle::default(),
                    material: Handle::default(),
                })
            }),
            regions: RollGrid2D::new(region_size as usize, region_size as usize, region_min),
            update_queue: BlockUpdateQueue::default(),
            lock_update_queue: false,
            update_modification_queue: Vec::new(),
            update_modification_map: HashMap::new(),
        }.initial_load()
    }

    pub fn move_center<C: Into<(i32, i32, i32)>>(&mut self, center: C) {
        let center: (i32, i32, i32) = center.into();
        let center: Coord = center.into();
        let padded_distance = self.render_distance + WORLD_SIZE_PAD as i32;
        let padded_size = padded_distance * 2;
        let (render_x, render_y, render_z) = calculate_center_offset(self.render_distance, center, Some(Self::WORLD_BOUNDS)).section_coord().xyz();
        let (chunk_x, chunk_z) = calculate_center_offset(padded_distance, center, Some(Self::WORLD_BOUNDS)).chunk_coord().xz();
        let region_min = calculate_region_min((chunk_x, chunk_z));
        self.save_world().expect("Failed to save the world");
        todo!()
    }

    pub fn save_world(&mut self) -> Result<()> {
        self.save_queue.drain().try_for_each(|coord| {
            println!("Saving {coord:?}");
            let (chunk_x, chunk_z) = coord.xz();
            let mut chunk = self.chunks.take((chunk_x, chunk_z)).expect("Chunk was None");
            // chunk.save_id = PoolId::NULL;
            let (region_x, region_z) = (chunk_x >> 5, chunk_z >> 5);
            let mut region = if let Some(region) = self.regions.take((region_x, region_z)) {
                region
            } else {
                RegionFile::open_or_create(self.subworld_directory.join(format!("{region_x}.{region_z}.rg")))?
            };
            let result = region.write_timestamped((chunk_x & 31, chunk_z & 31), chunk.edit_time, |writer| {
                chunk.write_to(writer)?;
                Ok(())
            })?;
            self.chunks.set((chunk_x, chunk_z), chunk);
            self.regions.set((region_x, region_z), region);
            Ok(())
        })
    }

    // fn save_chunk(&mut self, chunk_x: i32, chunk_z: i32, chunk: &mut Chunk) -> Result<()> {
    //     chunk.save_id = PoolId::NULL;
    //     let (region_x, region_z) = (chunk_x >> 5, chunk_z >> 5);
    //     let mut region = if let Some(region) = self.regions.take((region_x, region_z)) {
    //         region
    //     } else {
    //         RegionFile::open_or_create(self.subworld_directory.join(format!("{region_x}.{region_z}.rg")))?
    //     };
    //     let result = region.write_timestamped((chunk_x & 31, chunk_z & 31), chunk.edit_time, |writer| {
    //         chunk.write_to(writer)?;
    //         Ok(())
    //     })?;
    //     Ok(())
    // }

    /// This does not save the chunk!
    fn unload_chunk(&mut self, chunk: &mut Chunk) {
        chunk.sections.iter_mut().for_each(|section| {
            let dirty_id = section.dirty_id.swap(PoolId::NULL);
            if !dirty_id.null() {
                self.dirty_queue.remove(dirty_id);
            }
        });
        let save_id = chunk.save_id.swap(PoolId::NULL);
        if !save_id.null() {
            self.save_queue.remove(save_id);
        }
        chunk.unload(self);
    }

    /// This method assumes the chunk has already been unloaded with unload_chunk()
    fn load_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> Result<()> {
        let (region_x, region_z) = (chunk_x >> 5, chunk_z >> 5);
        let mut chunk = self.chunks.take((chunk_x, chunk_z)).expect("Chunk was None");
        let mut region = if let Some(region) = self.regions.take((region_x, region_z)) {
            region
        } else {
            let region_path = self.subworld_directory.join(format!("{region_x}.{region_z}.rg"));
            if region_path.is_file() {
                RegionFile::open_or_create(region_path)?
            } else {
                chunk.edit_time = Timestamp::new(0);
                self.chunks.set((chunk_x, chunk_z), chunk);
                return Ok(());
            }
        };
        let result = region.read((chunk_x & 31, chunk_z & 31), |reader| {
            chunk.read_from(reader, self)
        });
        let chunk_y = chunk.section_y();
        for i in 0..chunk.sections.len() {
            let section_y = chunk_y + i as i32;
            let section_coord = Coord::new(chunk_x, section_y, chunk_z);
            if self.render_chunks.bounds().contains(section_coord) 
            && chunk.sections[i].dirty_id.null(){
                let id = self.dirty_queue.insert(section_coord);
                chunk.sections[i].dirty_id = id;
            }
        }
        chunk.edit_time = region.get_timestamp((chunk_x & 31, chunk_z & 31));
        match result {
            Err(Error::ChunkNotFound) => {
                /* chunk.unload() */
                // chunk.unload(self);
            },
            Err(other) => {
                println!("{other:?}");
                panic!();
            },
            _ => (),
        }
        self.chunks.set((chunk_x, chunk_z), chunk);
        self.regions.set((region_x, region_z), region);
        Ok(())
    }
    
    fn initial_load(mut self) -> Self {
        self.chunks.bounds().iter().for_each(|(chunk_x, chunk_z)| {
            self.load_chunk(chunk_x, chunk_z);
        });
        self
    }

    fn mark_section_dirty(&mut self, section_coord: Coord) {
        let block_y = section_coord.y * 16;
        if !self.render_chunks.bounds().contains(section_coord)
        || block_y < WORLD_BOTTOM
        || block_y >= WORLD_TOP {
            return;
        }
        let Some(mut chunk) = self.chunks.take(section_coord.xz()) else {
            panic!("Chunk was None");
        };
        let section_index = (section_coord.y - chunk.section_y()) as usize;
        if chunk.sections[section_index].dirty_id.null() {
            chunk.sections[section_index].dirty_id = self.dirty_queue.insert(section_coord);
        }
        
        self.chunks.set(section_coord.xz(), chunk);
    }

    fn mark_modified(&mut self, chunk_coord: ChunkCoord) {
        let Some(mut chunk) = self.chunks.take(chunk_coord.xz()) else {
            panic!("Chunk was None");
        };
        if chunk.save_id.null() {
            chunk.save_id = self.save_queue.insert(chunk_coord);
        }
        self.chunks.set(chunk_coord.xz(), chunk);
    }

    pub fn offset(&self) -> Coord {
        let grid_offset = self.chunks.offset();
        Coord::new(
            grid_offset.0 * 16,
            0,
            grid_offset.1 * 16
        )
    }

    
    fn get_section(&self, section_coord: Coord) -> Option<&Section> {
        if section_coord.y < 0 {
            return None;
        }
        let chunk = self.chunks.get((section_coord.x, section_coord.z))?;
        let y = section_coord.y - chunk.block_offset.y / 16;
        if y < 0 || y as usize >= chunk.sections.len() {
            return None;
        }
        Some(&chunk.sections[y as usize])
    }

    
    fn get_section_mut(&mut self, section_coord: Coord) -> Option<&mut Section> {
        if section_coord.y < 0 {
            return None;
        }
        let chunk = self.chunks.get_mut((section_coord.x, section_coord.z))?;
        let y = section_coord.y - chunk.block_offset.y / 16;
        if y < 0 || y as usize >= chunk.sections.len() {
            return None;
        }
        Some(&mut chunk.sections[y as usize])
    }

    
    pub fn get_chunk(&self, chunk_coord: (i32, i32)) -> Option<&Chunk> {
        self.chunks.get(chunk_coord)
    }

    
    pub fn get_chunk_mut(&mut self, chunk_coord: (i32, i32)) -> Option<&mut Chunk> {
        self.chunks.get_mut(chunk_coord)
    }

    /// Calls a function on a block.
    pub fn call<T: Into<Tag>, C: Into<(i32, i32, i32)>, S: AsRef<str>>(&mut self, coord: C, function: S, arg: T) -> Tag {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        let state = self.get_block(coord);
        if state.is_air() {
            return Tag::Null;
        }
        state.block().call(self, coord, state, function.as_ref(), arg.into())
    }

    pub fn query<'a, C: Into<(i32, i32, i32)>, T: Query<'a>>(&'a self, coord: C) -> T::Output {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.render_bounds().contains(coord) {
            return T::default();
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.query::<T>(coord)
    }

    fn set_update_ref(&mut self, coord: Coord, value: UpdateRef) -> UpdateRef {
        if !self.render_bounds().contains(coord) {
            return UpdateRef::NULL;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.set_update_ref(coord, value)
    }

    fn get_update_ref(&self, coord: Coord) -> UpdateRef {
        if !self.render_bounds().contains(coord) {
            return UpdateRef::NULL;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_update_ref(coord)
    }

    pub fn get_block<C: Into<(i32, i32, i32)>>(&self, coord: C) -> Id {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return Id::AIR;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_block(coord)
    }

    pub fn set_block<C: Into<(i32, i32, i32)>, S: Into<Id>>(&mut self, coord: C, state: S) -> Id {
        let state: Id = state.into();
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        // Only set blocks within the render bounds, because light engine updates only happen in
        // the render bounds.
        // The problem is that darkness propagation might cause light to repropagation to overflow out of bounds
        // and we don't want that because it would invalidate the lightmap.
        if !self.render_bounds().contains(coord) {
            return Id::AIR;
        }
        let old = self.get_block(coord);
        if state == old {
            return old;
        }
        let (state, enable) = if state != Id::AIR {
            let mut place_context = PlaceContext::new(coord, state, old);
            state.block().on_place(self, &mut place_context);
            while place_context.changed {
                place_context.changed = false;
                let old_copy = place_context.old;
                place_context.old = place_context.replacement;
                place_context.data = None;
                place_context.replacement.block().on_place(self, &mut place_context);
            }
            if old == place_context.replacement {
                return old;
            }
            if old != Id::AIR {
                let old_block = old.block();
                self.delete_data_internal(coord, old);
                old_block.on_remove(self, coord, old, place_context.replacement);
            }
            if let Some(data) = place_context.data {
                self.set_data(coord, data);
            }
            (place_context.replacement, place_context.enable)
        } else {
            (state, None)
        };
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let change = {
            let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
            chunk.set_block(coord, state)
        };
        match change.change {
            StateChange::Unchanged => state,
            StateChange::Changed(old) => {
                let cur_ref = self.get_update_ref(coord);
                if cur_ref.null() && (matches!(enable, Some(true))
                || (!matches!(enable, Some(false)) && state.block().enable_on_place(self, coord, state))) {
                    let new_ref = self.update_queue.push(coord);
                    self.set_update_ref(coord, new_ref);
                } else if !cur_ref.null() {
                    self.set_update_ref(coord, UpdateRef::NULL);
                    self.update_queue.remove(cur_ref);
                }
                let block = state.block();
                let my_rotation = block.rotation(self, coord, state);
                let my_occluder = block.occluder(self, state);
                let neighbors = self.neighbors(coord);
                Direction::iter().for_each(|dir| {
                    let neighbor_dir = dir.invert();
                    let neighbor = neighbors[dir];
                    let neighbor_block = neighbor.block();
                    let neighbor_coord = coord + dir;
                    if neighbor != Id::AIR {
                        neighbor_block.neighbor_updated(self, neighbor_dir, neighbor_coord, coord, neighbor, state);
                    }
                    let neighbor_rotation = neighbor_block.rotation(self, neighbor_coord, neighbors[dir]);
                    let face_occluder = my_occluder.face(my_rotation.source_face(dir));
                    let neighbor_occluder = neighbor_block.occluder(self, neighbor).face(neighbor_rotation.source_face(neighbor_dir));
                    let neighbor_coord = coord + dir;
                    let my_angle = my_rotation.face_angle(dir);
                    let neighbor_angle = neighbor_rotation.face_angle(neighbor_dir);
                    if neighbor_occluder.occluded_by(face_occluder, neighbor_angle, my_angle) {
                        self.hide_face(neighbor_coord, neighbor_dir);
                    } else {
                        self.show_face(neighbor_coord, neighbor_dir);
                    }
                    if face_occluder.occluded_by(neighbor_occluder, my_angle, neighbor_angle) {
                        self.hide_face(coord, dir);
                    } else {
                        self.show_face(coord, dir);
                    }
                });
                self.mark_modified(coord.chunk_coord());
                if change.marked_dirty {
                    if self.render_bounds().contains(coord) {
                        let section_coord = coord.section_coord();
                        // self.dirty_sections.push(section_coord);
                        self.mark_section_dirty(section_coord);
                    }
                }
                old
            },
        }
    }

    pub fn occlusion<C: Into<(i32, i32, i32)>>(&self, coord: C) -> Occlusion {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return Occlusion::UNOCCLUDED;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.occlusion(coord)
    }

    pub fn face_visible<C: Into<(i32, i32, i32)>>(&self, coord: C, face: Direction) -> bool {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return true;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.face_visible(coord, face)
    }

    pub fn show_face<C: Into<(i32, i32, i32)>>(&mut self, coord: C, face: Direction) -> bool {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return true;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        let change = chunk.show_face(coord, face);
        if change.change {
            self.mark_modified(coord.chunk_coord());
        }
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                // self.dirty_sections.push(section_coord);
                self.mark_section_dirty(section_coord);
            }
        }
        change.change
    }

    pub fn hide_face<C: Into<(i32, i32, i32)>>(&mut self, coord: C, face: Direction) -> bool {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return true;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        let change = chunk.hide_face(coord, face);
        if !change.change {
            self.mark_modified(coord.chunk_coord());
        }
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                // self.dirty_sections.push(section_coord);
                self.mark_section_dirty(section_coord);
            }
        }
        change.change
    }

    pub fn get_block_light<C: Into<(i32, i32, i32)>>(&self, coord: C) -> u8 {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return 0;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_block_light(coord)
    }

    pub fn set_block_light<C: Into<(i32, i32, i32)>>(&mut self, coord: C, level: u8) -> LightChange {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return LightChange::default();
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        let change = chunk.set_block_light(coord, level);
        if change.change.changed() {
            self.mark_modified(coord.chunk_coord());
        }
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                // self.dirty_sections.push(section_coord);
                self.mark_section_dirty(section_coord);
            }
        }
        if change.change.new_max != change.change.old_max {
            let block = self.get_block(coord);
            if block != Id::AIR {
                block.block().light_updated(self, coord, change.change.old_max, change.change.new_max);
            }
        }
        change.change
    }

    pub fn get_sky_light<C: Into<(i32, i32, i32)>>(&self, coord: C) -> u8 {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return 0;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_sky_light(coord)
    }

    pub fn set_sky_light<C: Into<(i32, i32, i32)>>(&mut self, coord: C, level: u8) -> LightChange {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return LightChange::default();
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        let change = chunk.set_sky_light(coord, level);
        if change.change.changed() {
            self.mark_modified(coord.chunk_coord());
        }
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                // self.dirty_sections.push(section_coord);
                self.mark_section_dirty(section_coord);
            }
        }
        if change.change.new_max != change.change.old_max {
            let block = self.get_block(coord);
            if block != Id::AIR {
                block.block().light_updated(self, coord, change.change.old_max, change.change.new_max);
            }
        }
        change.change
    }

    pub fn get_data<C: Into<(i32, i32, i32)>>(&self, coord: C) -> Option<&Tag> {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return None;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_data(coord)
    }

    pub fn get_data_mut<C: Into<(i32, i32, i32)>>(&mut self, coord: C) -> Option<&mut Tag> {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return None;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_data_mut(coord)
    }

    pub fn get_or_insert_data<C: Into<(i32, i32, i32)>>(&mut self, coord: C, value: Tag) -> &mut Tag {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            panic!("Out of bounds.");
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        if self.get_data(coord).is_none() {
            self.mark_modified(coord.chunk_coord());
        }
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_or_insert_data(coord, value)
    }

    pub fn get_or_insert_data_with<C: Into<(i32, i32, i32)>, F: FnOnce() -> Tag>(&mut self, coord: C, f: F) -> &mut Tag {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            panic!("Out of bounds.");
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        if self.get_data(coord).is_none() {
            self.mark_modified(coord.chunk_coord());
        }
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_or_insert_data_with(coord, f)
    }

    pub fn take_data<C: Into<(i32, i32, i32)>>(&mut self, coord: C) -> Tag {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return Tag::Null;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        if let Some(data) = chunk.delete_data(coord) {
            self.mark_modified(coord.chunk_coord());
            data
        } else {
            Tag::Null
        }
    }

    pub fn delete_data<C: Into<(i32, i32, i32)>>(&mut self, coord: C) {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        if let Some(data) = chunk.delete_data(coord) {
            let state = self.get_block(coord);
            if !state.is_air() {
                state.block().on_data_delete(self, coord, state, data);
            }
            self.mark_modified(coord.chunk_coord());
        }
    }

    fn delete_data_internal<C: Into<(i32, i32, i32)>>(&mut self, coord: C, old_state: Id) {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        if let Some(data) = chunk.delete_data(coord) {
            if !old_state.is_air() {
                old_state.block().on_data_delete(self, coord, old_state, data);
            }
            self.mark_modified(coord.chunk_coord());
        }
    }

    pub fn set_data<C: Into<(i32, i32, i32)>, T: Into<Tag>>(&mut self, coord: C, tag: T) {
        let mut tag: Tag = tag.into();
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let state = self.get_block(coord);
        state.block().on_data_set(self, coord, state, &mut tag);
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        if let Some(data) = chunk.set_data(coord, tag) {
            if !state.is_air() {
                state.block().on_data_delete(self, coord, state, data);
            }
        }
        self.mark_modified(coord.chunk_coord());
    }

    
    pub fn enabled<C: Into<(i32, i32, i32)>>(&self, coord: C) -> bool {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if self.lock_update_queue {
            if let Some(index) = self.update_modification_map.get(&coord) {
                return self.update_modification_queue[*index as usize].1;
            }
        }
        !self.get_update_ref(coord).null()
    }

    pub fn set_enabled<C: Into<(i32, i32, i32)>>(&mut self, coord: C, enabled: bool) -> bool {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if self.lock_update_queue {
            let index = self.update_modification_queue.len() as u32;
            let current = self.update_modification_map.entry(coord).or_insert(index);
            if *current == index {
                self.update_modification_queue.push((coord, enabled));
                return !self.get_update_ref(coord).null();
            } else {
                return self.update_modification_queue[*current as usize].1.swap(enabled);
            }
        }
        if !self.bounds().contains(coord) {
            return false;
        }
        let state = self.get_block(coord);
        if state.is_air() {
            return false;
        }
        let cur_ref = self.get_update_ref(coord);
        // if cur_ref is null, check if 
        if enabled {
            // currently disabled, so enable it
            if cur_ref.null() {
                if state.is_air() {
                    return false;
                }
                self.mark_modified(coord.chunk_coord());
                let new_ref = self.update_queue.push(coord);
                self.set_update_ref(coord, new_ref);
                state.block().on_enabled_changed(self, coord, state, true);
                false
            } else {
                true
            }
        } else {
            if cur_ref.null() {
                false
            // currently enabled, so disable it
            } else {
                self.mark_modified(coord.chunk_coord());
                let cur_ref = self.set_update_ref(coord, UpdateRef::NULL);
                self.update_queue.remove(cur_ref);
                state.block().on_enabled_changed(self, coord, state, false);
                true
            }
        }
        
    }

    /// Enable a block, adding it to the update queue.
    pub fn enable<C: Into<(i32, i32, i32)>>(&mut self, coord: C) {
        self.set_enabled(coord, true);
    }


    /// Disable a block, removing it from the update queue if it's in the update queue.
    pub fn disable<C: Into<(i32, i32, i32)>>(&mut self, coord: C) {
        self.set_enabled(coord, false);
    }

    pub fn update(&mut self) {
        if self.lock_update_queue.swap(true) {
            panic!("World is already updating!");
        }
        self.update_modification_queue.clear();
        self.update_modification_map.clear();
        (0..self.update_queue.update_queue.len()).for_each(|i| {
            let coord = self.update_queue.update_queue[i].0;
            let state = self.get_block(coord);
            state.block().on_update(self, coord, state);
        });
        (0..self.update_modification_queue.len()).for_each(|i| {
            let (coord, enabled) = self.update_modification_queue[i];
            self.set_enabled(coord, enabled);
        });
        self.lock_update_queue = false;
    }

    pub fn height(&self, x: i32, z: i32) -> i32 {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        if let Some(chunk) = self.chunks.get((x, z)) {
            chunk.height(x, z)
        } else {
            WORLD_BOTTOM
        }
    }

    pub fn neighbors<C: Into<(i32, i32, i32)>>(&self, coord: C) -> Faces<Id> {
        cast_coord!(coord);
        use Direction::*;
        macro_rules! get_faces {
            ($(@$dir:expr),*) => {
                Faces::new(
                    $(
                        if let Some(next) = coord.checked_neighbor($dir) {
                            self.get_block(next)
                        } else {
                            Id::AIR
                        },
                    )*
                )
            };
        }
        get_faces!(
            @NegX,
            @NegY,
            @NegZ,
            @PosX,
            @PosY,
            @PosZ
        )
    }

    pub fn bounds(&self) -> Bounds3D {
        let bounds = self.chunks.bounds();
        let (min_x, min_z) = bounds.min;
        let (max_x, max_z) = bounds.max;
        let (min_x, min_z) = (
            min_x * 16,
            min_z * 16
        );
        let (maxx, maxz) = (
            max_x * 16,
            max_z * 16
        );
        let min_y = WORLD_BOTTOM;
        let max_y = WORLD_TOP;
        Bounds3D::new(
            (min_x, min_y, min_z),
            (maxx, max_y, maxz)
        )
    }

    pub fn render_bounds(&self) -> Bounds3D {
        let bounds = self.render_chunks.bounds();
        let (min_x, min_y, min_z) = bounds.min;
        let (max_x, max_y, max_z) = bounds.max;
        let (min_x, min_y, min_z) = (
            min_x * 16,
            min_y * 16,
            min_z * 16
        );
        let (max_x, max_y, max_z) = (
            max_x * 16,
            max_y * 16,
            max_z * 16
        );
        Bounds3D::new(
            (min_x, min_y, min_z),
            (max_x, max_y, max_z)
        )
    }

    pub fn dynamic_usage(&self) -> MemoryUsage {
        self.chunks.iter().map(|(_, chunk)| {
            let Some(chunk) = chunk else {
                panic!("Chunk was None.");
            };
            chunk.dynamic_usage()
        }).sum()
    }
}

pub struct MemoryUsage {
    pub used: usize,
    pub total: usize,
}

impl std::fmt::Display for MemoryUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Virtual: {} Used: {}", self.total, self.used)
    }
}

impl Sum for MemoryUsage {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(MemoryUsage::new(0,0), |mut usage, rhs| {
            MemoryUsage::new(usage.used + rhs.used, usage.total + rhs.total)
        })
    }
}

impl std::ops::Add<MemoryUsage> for MemoryUsage {
    type Output = MemoryUsage;
    fn add(self, rhs: MemoryUsage) -> Self::Output {
        Self {
            used: self.used + rhs.used,
            total: self.total + rhs.total,
        }
    }
}

impl MemoryUsage {
    pub fn new(used: usize, total: usize) -> Self {
        Self {
            used, total
        }
    }
}

pub struct RenderChunk {
    pub mesh: Handle<Mesh>,
    pub material: Handle<VoxelMaterial>,
}

pub struct PlaceContext {
    coord: Coord,
    replacement: Id,
    old: Id,
    data: Option<Tag>,
    changed: bool,
    enable: Option<bool>,
}

impl PlaceContext {
    pub fn new(coord: Coord, replacement: Id, old: Id) -> Self {
        Self {
            coord,
            replacement,
            old,
            data: None,
            changed: false,
            enable: None,
        }
    }

    pub fn replace(&mut self, state: Id) {
        self.replacement = state;
        self.changed = true;
    }

    pub fn set_data<T: Into<Tag>>(&mut self, data: T) {
        self.data = Some(data.into());
    }

    pub fn coord(&self) -> Coord {
        self.coord
    }

    pub fn old(&self) -> Id {
        self.old
    }

    pub fn replacement(&self) -> Id {
        self.replacement
    }

    pub fn enabled(&self) -> bool {
        matches!(self.enable, Some(true))
    }

    /// This is not the same as `!enabled()`!
    pub fn disabled(&self) -> bool {
        matches!(self.enable, Some(false))
    }

    pub fn enable(&mut self) {
        self.enable = Some(true);
    }

    pub fn disable(&mut self) {
        self.enable = Some(false);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enable.swap(Some(true));
    }
}