use std::{collections::VecDeque, iter::Sum};

use bevy::{asset::Handle, render::mesh::Mesh};
use super::occlusion::Occlusion;
use super::query::Query;
use rollgrid::{rollgrid2d::*, rollgrid3d::*};
use super::section::{LightChange, Section, StateChange};
use super::update::{BlockUpdateQueue, UpdateRef};

use crate::core::{math::grid::calculate_center_offset, voxel::{blocks::Id, coord::Coord, direction::Direction, engine::VoxelEngine, faces::Faces, rendering::voxelmaterial::VoxelMaterial}};
use crate::prelude::SwapVal;

use super::chunk::Chunk;

use crate::core::voxel::tag::Tag;

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
Query Engine
World Edit
*/

pub struct VoxelWorld {
    /// Determines if the render world has been initialized.
    pub initialized: bool,
    pub dirty_sections: Vec<Coord>,
    pub chunks: RollGrid2D<Chunk>,
    pub render_chunks: RollGrid3D<RenderChunk>,
    // pub regions: RollGrid2D<RegionFile>,
    pub update_queue: BlockUpdateQueue,
}

impl VoxelWorld {
    /// The maximum bounds of the world.
    const WORLD_BOUNDS: Bounds3D = Bounds3D {
        min: (i32::MIN, WORLD_BOTTOM, i32::MIN),
        max: (i32::MAX, WORLD_TOP, i32::MAX)
    };
    /// Create a new world centered at the specified block coordinate with the (chunk) render distance specified.
    /// The resulting width in chunks will be `render_distance * 2`.
    pub fn new(render_distance: u8, center: Coord) -> Self {
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
        let (render_x, render_y, render_z) = calculate_center_offset(render_distance as i32, center, Some(Self::WORLD_BOUNDS)).section_coord().xyz();
        Self {
            initialized: false,
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
            update_queue: BlockUpdateQueue::default(),
        }
    }

    #[inline(always)]
    pub fn offset(&self) -> Coord {
        let grid_offset = self.chunks.offset();
        Coord::new(
            grid_offset.0 * 16,
            0,
            grid_offset.1 * 16
        )
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    pub fn get_chunk(&self, chunk_coord: (i32, i32)) -> Option<&Chunk> {
        self.chunks.get(chunk_coord)
    }

    #[inline(always)]
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
                if change.marked_dirty {
                    if self.render_bounds().contains(coord) {
                        let section_coord = coord.section_coord();
                        self.dirty_sections.push(section_coord);
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
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                self.dirty_sections.push(section_coord);
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
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                self.dirty_sections.push(section_coord);
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
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                self.dirty_sections.push(section_coord);
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
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                self.dirty_sections.push(section_coord);
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

    pub fn get_or_insert_data<C: Into<(i32, i32, i32)>>(&mut self, coord: C, default: Tag) -> &mut Tag {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            panic!("Out of bounds.");
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_or_insert_data(coord, default)
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
    }

    pub fn set_enabled<C: Into<(i32, i32, i32)>>(&mut self, coord: C, enabled: bool) -> bool {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
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
            // currently enabled, so disable it (lol)
            } else {
                let cur_ref = self.set_update_ref(coord, UpdateRef::NULL);
                self.update_queue.remove(cur_ref);
                state.block().on_enabled_changed(self, coord, state, false);
                true
            }
        }
        
    }

    /// Enable a block, adding it to the update queue.
    #[inline(always)]
    pub fn enable<C: Into<(i32, i32, i32)>>(&mut self, coord: C) {
        self.set_enabled(coord, true);
    }


    /// Disable a block, removing it from the update queue if it's in the update queue.
    #[inline(always)]
    pub fn disable<C: Into<(i32, i32, i32)>>(&mut self, coord: C) {
        self.set_enabled(coord, false);
    }

    #[inline(always)]
    pub fn update(&mut self) {
        (0..self.update_queue.update_queue.len()).for_each(|i| {
            let coord = self.update_queue.update_queue[i].0;
            let state = self.get_block(coord);
            state.block().on_update(self, coord, state);
        });
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
    #[inline(always)]
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

    #[inline(always)]
    pub fn replace(&mut self, state: Id) {
        self.replacement = state;
        self.changed = true;
    }

    #[inline(always)]
    pub fn set_data<T: Into<Tag>>(&mut self, data: T) {
        self.data = Some(data.into());
    }

    #[inline(always)]
    pub fn coord(&self) -> Coord {
        self.coord
    }

    #[inline(always)]
    pub fn old(&self) -> Id {
        self.old
    }

    #[inline(always)]
    pub fn replacement(&self) -> Id {
        self.replacement
    }

    #[inline(always)]
    pub fn enabled(&self) -> bool {
        matches!(self.enable, Some(true))
    }

    /// This is not the same as `!enabled()`!
    #[inline(always)]
    pub fn disabled(&self) -> bool {
        matches!(self.enable, Some(false))
    }

    #[inline(always)]
    pub fn enable(&mut self) {
        self.enable = Some(true);
    }

    #[inline(always)]
    pub fn disable(&mut self) {
        self.enable = Some(false);
    }

    #[inline(always)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enable.swap(Some(true));
    }
}