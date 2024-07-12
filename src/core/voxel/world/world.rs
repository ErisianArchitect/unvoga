
use std::collections::VecDeque;

use bevy::{asset::Handle, render::mesh::Mesh};
use rollgrid::{rollgrid2d::*, rollgrid3d::*};

use crate::core::{math::grid::calculate_center_offset, voxel::{blocks::StateRef, coord::Coord, direction::Direction, engine::VoxelEngine, faces::Faces, rendering::voxelmaterial::VoxelMaterial}};

use super::chunk::{Chunk, LightChange, OcclusionFlags, Section, StateChange};

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
    chunks: RollGrid2D<Chunk>,
    render_chunks: RollGrid3D<RenderChunk>,
    dirty_sections: Vec<Coord>,
}

impl VoxelWorld {
    pub fn new(render_distance: usize, center: Coord) -> Self {
        let mut center = center;
        // clamp Y to world Y range
        center.y = center.y.min(WORLD_TOP).max(WORLD_BOTTOM);
        if render_distance + WORLD_SIZE_PAD > PADDED_WORLD_SIZE_MAX {
            panic!("Size greater than {PADDED_WORLD_SIZE_MAX} (PADDED_WORLD_SIZE_MAX)");
        }
        let pad_distance = (render_distance + WORLD_SIZE_PAD);
        let pad_size = pad_distance * 2;
        let render_size = render_distance * 2;
        // let pad_half = pad_distance as i32;
        // let render_half = render_distance as i32;
        // let (x, y, z) = (center.x, center.y, center.z);
        // let (offx, offy, offz) = (x - 8, y - 8, z - 8);
        // let pad_offset = (pad_half - 1) * 16;
        // let render_offset = (render_half - 1) * 16;
        // let (padx, pady, padz) = (
        //     (offx & !0xF) - pad_offset,
        //     (offy & !0xF) - pad_offset,
        //     (offz & !0xF) - pad_offset
        // );
        // let (rendx, rendy, rendz) = (
        //     (offx & !0xF) - render_offset,
        //     (offy & !0xF) - render_offset,
        //     (offz & !0xF) - render_offset
        // );
        // let clamp_top = WORLD_TOP - render_size as i32;
        // let rendy = rendy.min(clamp_top).max(WORLD_BOTTOM);
        let render_height = (render_distance * 2).max(WORLD_HEIGHT);
        let (chunk_x, chunk_z) = calculate_center_offset(pad_distance as i32, center).chunk_coord().xz();
        let (render_x, render_y, render_z) = calculate_center_offset(render_distance as i32, center).clamp_y(WORLD_BOTTOM, WORLD_TOP - render_height as i32).section_coord().xyz();
        Self {
            chunks: RollGrid2D::new_with_init(pad_size, pad_size, (chunk_x, chunk_z), |(x, z): (i32, i32)| {
                Some(Chunk::new(Coord::new(x * 16, WORLD_BOTTOM, z * 16)))
            }),
            render_chunks: RollGrid3D::new_with_init(render_size, render_size.min(WORLD_HEIGHT / 16), render_size, (render_x, render_y, render_z), |pos: Coord| {
                Some(RenderChunk {
                    mesh: Handle::default(),
                    material: Handle::default(),
                })
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

    pub fn get_block<C: Into<(i32, i32, i32)>>(&self, coord: C) -> StateRef {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if self.bounds().contains(coord) {
            let chunk_x = coord.x / 16;
            let chunk_z = coord.z / 16;
            if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
                chunk.get(coord)
            } else {
                StateRef::AIR
            }
        } else {
            StateRef::AIR
        }
    }

    pub fn get_block_light<C: Into<(i32, i32, i32)>>(&self, coord: C) -> u8 {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if self.bounds().contains(coord) {
            let chunk_x = coord.x / 16;
            let chunk_z = coord.z / 16;
            if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
                chunk.get_block_light(coord)
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn get_sky_light<C: Into<(i32, i32, i32)>>(&self, coord: C) -> u8 {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if self.bounds().contains(coord) {
            let chunk_x = coord.x / 16;
            let chunk_z = coord.z / 16;
            if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
                chunk.get_sky_light(coord)
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn occlusion<C: Into<(i32, i32, i32)>>(&self, coord: C) -> OcclusionFlags {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if self.bounds().contains(coord) {
            let chunk_x = coord.x / 16;
            let chunk_z = coord.z / 16;
            if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
                chunk.occlusion(coord)
            } else {
                OcclusionFlags::UNOCCLUDED
            }
        } else {
            OcclusionFlags::UNOCCLUDED
        }
    }

    pub fn face_visible<C: Into<(i32, i32, i32)>>(&self, coord: C, face: Direction) -> bool {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if self.bounds().contains(coord) {
            let chunk_x = coord.x / 16;
            let chunk_z = coord.z / 16;
            if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
                chunk.face_visible(coord, face)
            } else {
                true
            }
        } else {
            true
        }
    }

    pub fn height(&self, x: i32, z: i32) -> i32 {
        let chunk_x = x / 16;
        let chunk_z = z / 16;
        if let Some(chunk) = self.chunks.get((x, z)) {
            chunk.height(x, z)
        } else {
            WORLD_BOTTOM
        }
    }

    pub fn set_block<C: Into<(i32, i32, i32)>, S: Into<StateRef>>(&mut self, coord: C, state: S) -> StateRef {
        let state: StateRef = state.into();
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return StateRef::AIR;
        }
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
                let block = state.block();
                let my_rotation = block.block_rotation(state);
                if old != StateRef::AIR {
                    let old_block = old.block();
                    old_block.on_remove(self, coord, old, state);
                }
                if state != StateRef::AIR {
                    block.on_place(self, coord, old, state);
                }
                let my_occluder = block.occlusion_shapes(state);
                let neighbors = self.neighbors(coord);
                Direction::iter().for_each(|dir| {
                    let inv = dir.invert();
                    let neighbor = neighbors[dir];
                    let neighbor_block = neighbor.block();
                    let neighbor_rotation = neighbor_block.block_rotation(neighbors[dir]);
                    let face_occluder = &my_occluder[my_rotation.source_face(dir)];
                    let neighbor_occluder = &neighbor_block.occlusion_shapes(neighbor)[neighbor_rotation.source_face(inv)];
                    let neighbor_coord = coord + dir;
                    if neighbor_occluder.occluded_by(face_occluder) {
                        self.hide_face(neighbor_coord, inv);
                    } else {
                        self.show_face(neighbor_coord, inv);
                    }
                    if face_occluder.occluded_by(neighbor_occluder) {
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

    pub fn show_face<C: Into<(i32, i32, i32)>>(&mut self, coord: C, face: Direction) -> bool {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return true;
        }
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.show_face(coord, face)
        } else {
            return true;
        };
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
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.hide_face(coord, face)
        } else {
            return true;
        };
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                self.dirty_sections.push(section_coord);
            }
        }
        change.change
    }

    pub fn set_block_light<C: Into<(i32, i32, i32)>>(&mut self, coord: C, level: u8) -> LightChange {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return LightChange::default();
        }
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.set_block_light(coord, level)
        } else {
            return LightChange::default();
        };
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                self.dirty_sections.push(section_coord);
            }
        }
        if change.change.new_max != change.change.old_max {
            let block = self.get_block(coord);
            block.block().light_updated(self, coord, change.change.old_max, change.change.new_max);
        }
        change.change
    }

    pub fn set_sky_light<C: Into<(i32, i32, i32)>>(&mut self, coord: C, level: u8) -> LightChange {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return LightChange::default();
        }
        let chunk_x = coord.x / 16;
        let chunk_z = coord.z / 16;
        let change = if let Some(chunk) = self.chunks.get_mut((chunk_x, chunk_z)) {
            chunk.set_sky_light(coord, level)
        } else {
            return LightChange::default();
        };
        if change.marked_dirty {
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                self.dirty_sections.push(section_coord);
            }
        }
        if change.change.new_max != change.change.old_max {
            let block = self.get_block(coord);
            block.block().light_updated(self, coord, change.change.old_max, change.change.new_max);
        }
        change.change
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

    pub fn dynamic_usage(&self) -> usize {
        self.chunks.iter().map(|(_, chunk)| {
            let Some(chunk) = chunk else {
                return 0;
            };
            chunk.dynamic_usage()
        }).sum()
    }
}

struct RenderChunk {
    pub mesh: Handle<Mesh>,
    pub material: Handle<VoxelMaterial>,
}

#[cfg(test)]
mod tests {
    use crate::{blockstate, core::{math::coordmap::Rotation, voxel::{block::Block, blocks::{self, StateRef}, blockstate::State, coord::Coord, direction::Direction, faces::Faces, occlusion_shape::OcclusionShape, world::world::WORLD_TOP}}};

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

    #[test]
    fn world_test() {
        println!("World Test");
        let mut world = VoxelWorld::new(32, Coord::new(0, -10000, 0));
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

            fn occlusion_shapes(&self, state: StateRef) -> &Faces<OcclusionShape> {
                const SHAPE: Faces<OcclusionShape> = Faces::new(
                    OcclusionShape::Full,
                    OcclusionShape::Full,
                    OcclusionShape::Full,
                    OcclusionShape::None,
                    OcclusionShape::None,
                    OcclusionShape::None
                );
                &SHAPE
            }

            fn default_state(&self) -> crate::core::voxel::blockstate::BlockState {
                blockstate!(rotated, rotation=Rotation::new(Direction::PosY, 0))
            }

            fn block_rotation(&self, state: StateRef) -> Rotation {
                if let Some(&State::Rotation(rotation)) = state.get_property("rotation") {
                    rotation
                } else {
                    Rotation::default()
                }
            }
        }
        blocks::register_block(DirtBlock);
        blocks::register_block(RotatedBlock);
        let dirt = blockstate!(dirt).register();
        let rot1 = blockstate!(rotated, rotation=Rotation::new(Direction::PosZ, 1)).register();
        let rot2 = blockstate!(rotated, rotation=Rotation::new(Direction::PosZ, 3)).register();
        for y in 0..3 {
            for z in 0..3 {
                for x in 0..3 {
                    world.set_block((x, y, z), dirt);
                }
            }
        }
        world.set_block((1, 1, 1), rot2);
        let flags = world.occlusion((1, 1, 1));
        println!("NegX: {}", flags.neg_x());
        println!("NegY: {}", flags.neg_y());
        println!("NegZ: {}", flags.neg_z());
        println!("PosX: {}", flags.pos_x());
        println!("PosY: {}", flags.pos_y());
        println!("PosZ: {}", flags.pos_z());
        let height = world.height(0, 0);
        println!("Dynamic Memory Usage: {}", world.dynamic_usage());
        println!("Height: {height}");
        println!(" World Bounds: {:?}", world.bounds());
        println!("Render Bounds: {:?}", world.render_bounds());
        println!("  Block Count: {}", world.bounds().volume());
        // for y in 0..16 {
        //     for z in 0..16 {
        //         for x in 0..16 {
        //             world.set_block((x, y, z), dirt);
        //             world.set_sky_light((x, y, z), 7);
        //             world.set_block_light((x, y, z), 13);
        //         }
        //     }
        // }
        let usage = world.dynamic_usage();
        println!("Dynamic Memory Usage: {}", usage);
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    world.set_block((x, y, z), StateRef::AIR);
                    world.set_sky_light((x, y, z), 0);
                    world.set_block_light((x, y, z), 0);
                }
            }
        }
        world.set_block((0, 0, 0), dirt);
        let usage = world.dynamic_usage();
        // assert_eq!(usage, 0);
        println!("Dynamic Memory Usage: {}", usage);
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
