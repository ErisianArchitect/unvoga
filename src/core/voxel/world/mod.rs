pub mod chunk;
pub mod section;
pub mod occlusion;
pub mod heightmap;
pub mod dirty;
pub mod chunkprovider;
pub mod chunkcoord;
pub mod blockdata;
pub mod update;

use std::{collections::VecDeque, iter::Sum};

use bevy::{asset::Handle, render::mesh::Mesh};
use occlusion::Occlusion;
use rollgrid::{rollgrid2d::*, rollgrid3d::*};
use section::{LightChange, Section, StateChange};
use update::BlockUpdateQueue;

use crate::core::{math::grid::calculate_center_offset, voxel::{blocks::StateRef, coord::Coord, direction::Direction, engine::VoxelEngine, faces::Faces, rendering::voxelmaterial::VoxelMaterial}};

use chunk::Chunk;

use super::tag::Tag;

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

pub struct VoxelWorld {
    /// Determines if the render world has been initialized.
    initialized: bool,
    dirty_sections: Vec<Coord>,
    chunks: RollGrid2D<Chunk>,
    render_chunks: RollGrid3D<RenderChunk>,
    update_queue: BlockUpdateQueue,
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

    pub fn message<T: Into<Tag>, C: Into<(i32, i32, i32)>>(&mut self, coord: C, message: T) -> Tag {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        let state = self.get_block(coord);
        if state.is_air() {
            return Tag::Null;
        }
        state.block().message(self, coord, state, message.into())
    }

    pub fn get_block<C: Into<(i32, i32, i32)>>(&self, coord: C) -> StateRef {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return StateRef::AIR;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_block(coord)
    }

    pub fn set_block<C: Into<(i32, i32, i32)>, S: Into<StateRef>>(&mut self, coord: C, state: S) -> StateRef {
        let state: StateRef = state.into();
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        // Only set blocks within the render bounds, because light engine updates only happen in
        // the render bounds.
        // The problem is that darkness propagation might cause light to repropagation to overflow out of bounds
        // and we don't want that because it would invalidate the lightmap.
        if !self.render_bounds().contains(coord) {
            return StateRef::AIR;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        let change = chunk.set_block(coord, state);
        match change.change {
            StateChange::Unchanged => state,
            StateChange::Changed(old) => {
                let block = state.block();
                let my_rotation = block.rotation(state);
                if old != StateRef::AIR {
                    let old_block = old.block();
                    self.delete_data_internal(coord, old);
                    old_block.on_remove(self, coord, old, state);
                }
                if state != StateRef::AIR {
                    block.on_place(self, coord, old, state);
                }
                let my_occluder = block.occluder(state);
                let neighbors = self.neighbors(coord);
                Direction::iter().for_each(|dir| {
                    let neighbor_dir = dir.invert();
                    let neighbor = neighbors[dir];
                    let neighbor_block = neighbor.block();
                    if neighbor != StateRef::AIR {
                        neighbor_block.neighbor_updated(self, neighbor_dir, coord + dir, coord, neighbor, state);
                    }
                    let neighbor_rotation = neighbor_block.rotation(neighbors[dir]);
                    let face_occluder = my_occluder.face(my_rotation.source_face(dir));
                    let neighbor_occluder = neighbor_block.occluder(neighbor).face(neighbor_rotation.source_face(neighbor_dir));
                    let neighbor_coord = coord + dir;
                    let my_angle = my_rotation.face_angle(dir);
                    let neighbor_angle = neighbor_rotation.face_angle(neighbor_dir);
                    // println!("{coord} {dir:?} -> {:?} {:?}", my_rotation.source_face(dir), neighbor_rotation.source_face(neighbor_dir));
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
            if block != StateRef::AIR {
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
            if block != StateRef::AIR {
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
                state.block().data_deleted(self, coord, state, data);
            }
        }
    }

    fn delete_data_internal<C: Into<(i32, i32, i32)>>(&mut self, coord: C, old_state: StateRef) {
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
                old_state.block().data_deleted(self, coord, old_state, data);
            }
        }
    }

    pub fn set_data<C: Into<(i32, i32, i32)>>(&mut self, coord: C, tag: Tag) {
        let mut tag = tag;
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let state = self.get_block(coord);
        state.block().data_set(self, coord, state, &mut tag);
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        if let Some(data) = chunk.set_data(coord, tag) {
            if !state.is_air() {
                state.block().data_deleted(self, coord, state, data);
            }
        }
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

    pub fn neighbors<C: Into<(i32, i32, i32)>>(&self, coord: C) -> Faces<StateRef> {
        cast_coord!(coord);
        use Direction::*;
        macro_rules! get_faces {
            ($(@$dir:expr),*) => {
                Faces::new(
                    $(
                        if let Some(next) = coord.checked_neighbor($dir) {
                            self.get_block(next)
                        } else {
                            StateRef::AIR
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
    used: usize,
    total: usize,
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

struct RenderChunk {
    pub mesh: Handle<Mesh>,
    pub material: Handle<VoxelMaterial>,
}

#[cfg(test)]
mod tests {
    use crate::{blockstate, core::{math::coordmap::Rotation, voxel::{block::Block, blocks::{self, StateRef}, blockstate::StateValue, coord::Coord, direction::Direction, faces::Faces, occluder::Occluder, occlusion_shape::OcclusionShape, tag::Tag, world::{occlusion::Occlusion, WORLD_TOP}}}};

    use super::VoxelWorld;

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

        fn on_place(
                &self,
                world: &mut VoxelWorld,
                coord: Coord,
                old: StateRef,
                new: StateRef,
            ) {
                // world.set_block(coord, StateRef::AIR);
                println!("dirt placed: {new}");
        }

        fn default_state(&self) -> crate::core::voxel::blockstate::BlockState {
            blockstate!(dirt)
        }
    }
    struct RotatedBlock;
    impl Block for RotatedBlock {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn name(&self) -> &str {
            "rotated"
        }

        fn occluder(&self, state: StateRef) -> &Occluder {
            const SHAPE: Occluder = Occluder::new(
                OcclusionShape::Full,
                OcclusionShape::Full,
                OcclusionShape::Full,
                OcclusionShape::Empty,
                OcclusionShape::Empty,
                OcclusionShape::Empty
            );
            &SHAPE
        }

        fn default_state(&self) -> crate::core::voxel::blockstate::BlockState {
            blockstate!(rotated, rotation=Rotation::new(Direction::PosY, 0))
        }

        fn rotation(&self, state: StateRef) -> Rotation {
            if let Some(&StateValue::Rotation(rotation)) = state.get_property("rotation") {
                rotation
            } else {
                Rotation::default()
            }
        }
        fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: StateRef, neighbor_state: StateRef) {
            println!("Neighbor Updated(coord = {coord:?}, neighbor_coord = {neighbor_coord:?}, neighbor_state = {neighbor_state})");
        }
    }
    struct DebugBlock;
    impl Block for DebugBlock {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn name(&self) -> &str {
            "debug"
        }
        fn default_state(&self) -> crate::core::voxel::blockstate::BlockState {
            blockstate!(debug)
        }
        fn on_place(&self, world: &mut VoxelWorld, coord: Coord, old: StateRef, new: StateRef) {
            println!("On Place {coord} old = {old} new = {new}");
            if matches!(new["withdata"], StateValue::Bool(true)) {
                println!("Adding data...");
                world.set_data(coord, Tag::from("The quick brown fox jumps over the lazy dog."));
            }
        }
        fn on_remove(&self, world: &mut VoxelWorld, coord: Coord, old: StateRef, new: StateRef) {
            println!("On Remove {coord} old = {old} new = {new}");
        }
        fn data_deleted(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, data: Tag) {
            println!("Data Deleted {coord} state = {state} data = {data:?}");
        }
        fn light_updated(&self, world: &mut VoxelWorld, coord: Coord, old_level: u8, new_level: u8) {
            println!("Light Updated {coord} old = {old_level} new = {new_level}");
        }
        fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: StateRef, neighbor_state: StateRef) {
            println!("Neighbor Updated {coord} -> {neighbor_coord} {state} -> {neighbor_state}");
        }
    }

    #[test]
    fn world_test() {
        println!("World Test");
        let mut world = VoxelWorld::new(32, Coord::new(0, -10000, 0));
        blocks::register_block(DirtBlock);
        blocks::register_block(RotatedBlock);
        blocks::register_block(DebugBlock);
        println!(" World Bounds: {:?}", world.bounds());
        println!("Render Bounds: {:?}", world.render_bounds());
        println!("  Block Count: {}", world.bounds().volume());
        let air = StateRef::AIR;
        let debug = blockstate!(debug).register();
        let debug_data = blockstate!(debug, withdata = true).register();
        let dirt = blockstate!(dirt).register();
        let rot1 = blockstate!(rotated, rotation=Rotation::new(Direction::PosZ, 1)).register();
        let rot2 = blockstate!(rotated, rotation=Rotation::new(Direction::PosZ, 3)).register();
        
        itertools::iproduct!(15..16, 0..1, 15..16).for_each(|(y, z, x)| {
            world.set_block((x, y, z), debug_data);
        });
        world.set_block((15, 15, 15), debug_data);

        itertools::iproduct!(0..16, 0..16, 0..16).for_each(|(y, z, x)| {
            world.set_block((x, y, z), air);
        });
        itertools::iproduct!(0..16, 0..16, 0..16).for_each(|(y, z, x)| {
            let faces = world.occlusion((x, y, z));
            if faces != Occlusion::UNOCCLUDED {
                println!("Occluded at ({x:2}, {y:2}, {z:2})");
            }
        });
        let usage = world.dynamic_usage();
        println!("Memory: {usage}");

        // world.set_block((1, 1, 1), rot2);
        // println!("Block at (1, 1, 1): {}", world.get((1, 1, 1)));
        // let flags = world.occlusion((1, 1, 1));
        // println!("Occlusion at (1, 1, 1) = {flags}");
        // let height = world.height(0, 0);
        // println!("Dynamic Memory Usage: {}", world.dynamic_usage());
        // println!("Height: {height}");
        // for y in 0..16 {
        //     for z in 0..16 {
        //         for x in 0..16 {
        //             world.set_block((x, y, z), dirt);
        //             world.set_sky_light((x, y, z), 7);
        //             world.set_block_light((x, y, z), 13);
        //         }
        //     }
        // }
        // let usage = world.dynamic_usage();
        // println!("Dynamic Memory Usage: {}", usage);
        // for y in 0..16 {
        //     for z in 0..16 {
        //         for x in 0..16 {
        //             world.set((x, y, z), StateRef::AIR);
        //             world.set_sky_light((x, y, z), 0);
        //             world.set_block_light((x, y, z), 0);
        //         }
        //     }
        // }
        // world.set((0, 0, 0), dirt);
        // let usage = world.dynamic_usage();
        // assert_eq!(usage, 0);
        // println!("Dynamic Memory Usage: {}", usage);
        return;

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