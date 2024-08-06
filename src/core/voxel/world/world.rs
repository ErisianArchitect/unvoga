#![allow(unused)]
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use itertools::Itertools;
use tap::{Tap, TapFallible};
use std::path::{Path, PathBuf};
use std::{collections::VecDeque, iter::Sum};

use bevy::{asset::Handle, render::mesh::Mesh};
use hashbrown::HashMap;
use super::chunkcoord::ChunkCoord;
use super::externevent::ExternEvent;
use super::occlusion::Occlusion;
use super::query::{BlockLight, VoxelQuery, SkyLight};
use rollgrid::{rollgrid2d::*, rollgrid3d::*};
use super::section::{LightChange, Section, StateChange};
use super::update::{BlockUpdateQueue, UpdateRef};

use crate::core::collections::objectpool::{ObjectPool, PoolId};
use crate::core::math::aabb::AABB;
use crate::core::math::grid::{calculate_region_min, calculate_region_requirement};
use crate::core::util::lend::Lend;
use crate::core::voxel::procgen::worldgenerator::WorldGenerator;
use crate::core::voxel::region::regionfile::RegionFile;
use crate::core::voxel::region::timestamp::Timestamp;
use crate::core::voxel::rendering::meshbuilder::MeshBuilder;
use crate::core::{math::grid::calculate_center_offset, voxel::{blocks::Id, coord::Coord, direction::Direction, engine::VoxelEngine, faces::Faces, rendering::voxelmaterial::VoxelMaterial}};
use crate::prelude::{f32_not_zero, ResultExtension, SwapVal};

use super::chunk::Chunk;

use crate::core::voxel::tag::Tag;
use crate::core::error::*;

// Make sure this value is always a multiple of 16 and
// preferably a multiple of 64.
// pub const WORLD_HEIGHT: usize = 320;
// pub const WORLD_BOTTOM: i32 = -160;
pub const WORLD_HEIGHT: usize = 640;
pub const WORLD_BOTTOM: i32 = -400;
pub const WORLD_TOP: i32 = WORLD_BOTTOM + WORLD_HEIGHT as i32;
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MoveRenderChunkMarker;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldGenMarker;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoadChunkMarker;


#[derive(Debug, Component)]
pub struct RenderChunkMarker;

#[derive(Resource)]
pub struct VoxelWorld {
    pub array_texture: Handle<Image>,
    /// Determines if the render world has been initialized.
    // pub initialized: bool,
    // pub dirty_sections: Vec<Coord>,
    pub dirty_queue: Lend<ObjectPool<Coord, DirtyIdMarker>>,
    pub save_queue: ObjectPool<ChunkCoord, SaveIdMarker>,
    pub chunks: Lend<RollGrid2D<Chunk>>,
    pub render_chunks: Lend<RollGrid3D<RenderChunk>>,
    // I gotta figure out grid positioning for the loaded region files.
    // I can give it a buffer of 1 region so that there's room for the
    // world to move around without overflow.
    // I have an idea that involes rounding up to the next multiple of 32.
    // (n + 30) & -32
    // You'll have to do some hacky stuff to make -32u32.
    pub regions: Lend<RollGrid2D<RegionFile>>,
    pub update_queue: BlockUpdateQueue,
    pub lock_update_queue: bool,
    /// (Coord, new)
    pub update_modification_queue: Vec<(Coord, bool)>,
    /// The value is the index in the update_modification_queue where
    /// the modification is stored.
    pub update_modification_map: HashMap<Coord, u32>,
    pub world_directory: PathBuf,
    pub subworld_directory: PathBuf,
    pub move_render_chunk_queue: Lend<ObjectPool<Coord, MoveRenderChunkMarker>>,
    pub render_distance: i32,
    pub worldgen_queue: Lend<ObjectPool<(i32, i32), WorldGenMarker>>,
    pub load_queue: Lend<ObjectPool<(i32, i32), LoadChunkMarker>>,
    pub world_generator: Option<Box<dyn WorldGenerator>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastResult {
    pub hit_point: Vec3,
    pub coord: Coord,
    pub direction: Option<Direction>,
    pub id: Id,
}

impl RaycastResult {
    pub const fn new(hit_point: Vec3, coord: Coord, direction: Option<Direction>, id: Id) -> Self {
        Self {
            hit_point,
            coord,
            direction,
            id
        }
    }
}

impl VoxelWorld {
    /// The maximum bounds of the world.
    pub const WORLD_BOUNDS: Bounds3D = Bounds3D {
        min: (i32::MIN, WORLD_BOTTOM, i32::MIN),
        max: (i32::MAX, WORLD_TOP, i32::MAX)
    };
    /// Open or create a world centered at the specified block coordinate with the (chunk) render distance specified.
    /// The resulting width in chunks will be `render_distance * 2`.
    pub fn open<P: AsRef<Path>, C: Into<(i32, i32, i32)>>(
            directory: P,
            render_distance: u8,
            center: C,
            array_texture: Handle<Image>,
            commands: &mut Commands,
            meshes: &mut ResMut<Assets<Mesh>>,
            materials: &mut ResMut<Assets<VoxelMaterial>>,
            generator: Option<Box<dyn WorldGenerator>>,
        ) -> Self {
            
        let center: (i32, i32, i32) = center.into();
        let center: Coord = center.into();
        let mut center = center;
        if render_distance as usize + WORLD_SIZE_PAD > PADDED_WORLD_SIZE_MAX {
            panic!("Size greater than {PADDED_WORLD_SIZE_MAX} (PADDED_WORLD_SIZE_MAX)");
        }
        let pad_distance = (render_distance as usize + WORLD_SIZE_PAD);
        let pad_size = pad_distance * 2;
        let render_size = render_distance as usize * 2;
        let render_height = render_size.min(WORLD_HEIGHT >> 4);
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
        let mut load_queue = Lend::new(ObjectPool::new());
        Self {
            array_texture: array_texture.clone(),
            subworld_directory: main_world,
            dirty_queue: Lend::new(ObjectPool::new()),
            save_queue: ObjectPool::new(),
            render_distance: render_distance as i32,
            world_directory: directory.to_owned(),
            chunks: Lend::new(RollGrid2D::new_with_init(pad_size, pad_size, (chunk_x, chunk_z), |(x, z): (i32, i32)| {
                Some(Chunk::new(Coord::new(x * 16, WORLD_BOTTOM, z * 16)).tap_mut(|chunk| {
                    chunk.load_id = load_queue.insert((x, z));
                }))
            })),
            render_chunks: Lend::new(RollGrid3D::new(render_size, render_height, render_size, (render_x, render_y, render_z))),
            regions: Lend::new(RollGrid2D::new(region_size as usize, region_size as usize, region_min)),
            update_queue: BlockUpdateQueue::default(),
            lock_update_queue: false,
            update_modification_queue: Vec::new(),
            update_modification_map: HashMap::new(),
            move_render_chunk_queue: Lend::new(ObjectPool::new()),
            worldgen_queue: Lend::new(ObjectPool::new()),
            load_queue,
            world_generator: generator,
        }
        // .initial_load()
    }
    
    fn initial_load(mut self) -> Self {
        self.chunks.bounds().iter().try_for_each(|(chunk_x, chunk_z)| {
            let (region_x, region_z) = (chunk_x >> 5, chunk_z >> 5);
            let mut chunk = self.chunks.take((chunk_x, chunk_z)).expect("Chunk was None");
            chunk.unload(&mut self);
            let mut region = if let Some(region) = self.regions.take((region_x, region_z)) {
                region
            } else {
                let region_path = self.get_region_path(region_x, region_z);
                if region_path.is_file() {
                    RegionFile::open_or_create(region_path)?
                } else {
                    chunk.edit_time = Timestamp::new(0);
                    self.chunks.set((chunk_x, chunk_z), chunk);
                    return Result::Ok(());
                }
            };
            let result = region.read((chunk_x & 31, chunk_z & 31), |reader| {
                chunk.read_from(reader, &mut self)
            });
            match result {
                Err(Error::ChunkNotFound) => {
                    /* chunk.unload() */
                    // chunk.unload(self);
                },
                other => {
                    other?;
                },
                _ => (),
            }
            let chunk_y = chunk.section_y();
            // TODO: There's a better way to do this, lol
            let render_y_min = self.render_chunks.bounds().y_min();
            let render_y_max = self.render_chunks.bounds().y_max();
            for y in render_y_min..render_y_max {
                let chunk_bottom = chunk.block_offset.y >> 4;
                let diff = render_y_min - chunk_bottom;
                let yi = y - render_y_min;
                let i = (y - chunk_bottom) as usize;
                let section_coord = Coord::new(chunk_x, y, chunk_z);
                if !self.render_chunks.bounds().contains(section_coord) {
                    continue;
                }
                // this probably isn't necessary.
                // self.dirty_queue.remove(chunk.sections[i].dirty_id.swap_null());
                if chunk.sections[i].blocks.is_some() {
                    chunk.sections[i].dirty_id = self.dirty_queue.insert(section_coord);
                    chunk.sections[i].light_dirty.mark();
                    chunk.sections[i].blocks_dirty.mark();
                    chunk.sections[i].section_dirty.mark();
                }
            }
            chunk.edit_time = region.get_timestamp((chunk_x & 31, chunk_z & 31));
            self.chunks.set((chunk_x, chunk_z), chunk);
            self.regions.set((region_x, region_z), region);
            Ok(())
        });
        self
    }

    pub fn render_bounds_aabb(&self) -> AABB {
        let render_bounds = self.render_bounds();
        let (minx, miny, minz) = render_bounds.min;
        let (minx, miny, minz) = (minx as f32, miny as f32, minz as f32);
        let (maxx, maxy, maxz) = render_bounds.max;
        let (maxx, maxy, maxz) = (maxx as f32, maxy as f32, maxz as f32);
        AABB::new(vec3(minx, miny, minz), vec3(maxx, maxy, maxz))
    }

    pub fn raycast_filter<F: FnMut(&Self, Coord, Id) -> bool>(&self, ray: Ray3d, max_distance: f32, filter: F) -> Option<RaycastResult> {
        let mut filter = filter;
        let stepx = if ray.direction.x > 0.0 {
            Some(1i32)
        } else if ray.direction.x < 0.0 {
            Some(-1i32)
        } else {
            None
        };
        let stepy = if ray.direction.y > 0.0 {
            Some(1i32)
        } else if ray.direction.y < 0.0 {
            Some(-1i32)
        } else {
            None
        };
        let stepz = if ray.direction.z > 0.0 {
            Some(1i32)
        } else if ray.direction.z < 0.0 {
            Some(-1i32)
        } else {
            None
        };
        let mut current = Coord::new(
            ray.origin.x.floor() as i32,
            ray.origin.y.floor() as i32,
            ray.origin.z.floor() as i32
        );
        let dirfrac = AABB::calc_dirfrac(ray);
        let id = self.get_block(current);
        if filter(self, current, id) {
            let orientation = id.block().orientation(self, current, id);
            if let Some(dist) = id.block().raycast(ray, self, current, id, orientation) {
                return Some(RaycastResult::new(ray.origin + (ray.direction * dist), current, None, id));
            }
        }
        let mut iterations = 0u64;
        let max_iterations = max_distance as u64;
        loop {
            iterations += 1;
            if iterations > max_iterations {
                return None;
            }
            if let Some(stepx) = stepx {
                let xcoord = (
                    current.x + stepx,
                    current.y,
                    current.z
                );
                let aabb = AABB::voxel(xcoord);
                if let Some(dist) = aabb.intersects_frac(ray, dirfrac) {
                    if dist > max_distance {
                        return None;
                    }
                    let id = self.get_block(xcoord);
                    let xcoord = Coord::from(xcoord);
                    if filter(self, xcoord, id) {
                        let orientation = id.block().orientation(self, xcoord, id);
                        if let Some(dist) = id.block().raycast(ray, self, current, id, orientation) {
                            let direction = match stepx {
                                1 => Direction::NegX,
                                -1 => Direction::PosX,
                                _ => unreachable!()
                            };
                            return Some(RaycastResult::new(ray.origin + (ray.direction * dist), xcoord, Some(direction), id));
                        }
                    }
                    current.x += stepx;
                    continue;
                }
            }
            if let Some(stepz) = stepz {
                let stepcoord = (
                    current.x,
                    current.y,
                    current.z + stepz
                );
                let aabb = AABB::voxel(stepcoord);
                if let Some(dist) = aabb.intersects_frac(ray, dirfrac) {
                    if dist > max_distance {
                        return None;
                    }
                    let id = self.get_block(stepcoord);
                    let stepcoord = Coord::from(stepcoord);
                    if filter(self, stepcoord, id) {
                        let orientation = id.block().orientation(self, stepcoord, id);
                        if let Some(dist) = id.block().raycast(ray, self, current, id, orientation) {
                            let direction = match stepz {
                                1 => Direction::NegZ,
                                -1 => Direction::PosZ,
                                _ => unreachable!()
                            };
                            return Some(RaycastResult::new(ray.origin + (ray.direction * dist), stepcoord, Some(direction), id));
                        }
                    }
                    current.z += stepz;
                    continue;
                }
            }
            if let Some(stepy) = stepy {
                let stepcoord = (
                    current.x,
                    current.y + stepy,
                    current.z
                );
                let aabb = AABB::voxel(stepcoord);
                if let Some(dist) = aabb.intersects_frac(ray, dirfrac) {
                    if dist > max_distance {
                        return None;
                    }
                    let id = self.get_block(stepcoord);
                    let stepcoord = Coord::from(stepcoord);
                    if filter(self, stepcoord, id) {
                        let orientation = id.block().orientation(self, stepcoord, id);
                        if let Some(dist) = id.block().raycast(ray, self, current, id, orientation) {
                            let direction = match stepy {
                                1 => Direction::NegY,
                                -1 => Direction::PosY,
                                _ => unreachable!()
                            };
                            return Some(RaycastResult::new(ray.origin + (ray.direction * dist), stepcoord, Some(direction), id));
                        }
                    }
                    current.y += stepy;
                    continue;
                }
            }
        }
        None
        // let mut filter = filter;
        // let render_bounds = self.render_bounds_aabb();
        // let ray = if render_bounds.contains(ray.origin) {
        //     ray
        // } else {
        //     let origin = render_bounds.intersection_point(ray)?;
        //     Ray3d::new(origin, ray.direction.into())
        // };
        // let (mut x, mut y, mut z) = (
        //     ray.origin.x.floor(),
        //     ray.origin.y.floor(),
        //     ray.origin.z.floor()
        // );
        // let make_coord = |x: f32, y: f32, z: f32| {
        //     let (x, y, z) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
        //     Coord::new(x, y, z)
        // };
        // let mut coord = make_coord(x, y, z);
        // let ray_end = ray.origin + (ray.direction * max_distance);
        // let mut last_coord = make_coord(
        //     ray_end.x,
        //     ray_end.y,
        //     ray_end.z
        // );
        // let state = self.get_block(coord);
        // if filter(self, coord, state) {
        //     let mut current = state;
        //     let orientation = current.block().orientation(self, coord, current);
        //     if current != Id::AIR && current.block().raycast(self, coord, current, orientation) {
        //         return Some((coord, current));
        //     }
        // }
        // let mut stepx = if ray.direction.x > 0.0 { 1.0f32 } else if ray.direction.x < 0.0 { -1.0f32 } else { 0.0 };
        // let mut stepy = if ray.direction.y > 0.0 { 1.0f32 } else if ray.direction.y < 0.0 { -1.0f32 } else { 0.0 };
        // let mut stepz = if ray.direction.z > 0.0 { 1.0f32 } else if ray.direction.z < 0.0 { -1.0f32 } else { 0.0 };
        // let next_vox_bound_x = x + stepx;
        // let next_vox_bound_y = y + stepy;
        // let next_vox_bound_z = z + stepz;
        // let (mut tmaxx, tdeltax) = if f32_not_zero(ray.direction.x) {
        //     (
        //         (next_vox_bound_x - ray.origin.x) / ray.direction.x,
        //         1.0 / ray.direction.x/*  * stepx */
        //     )
        // } else {
        //     (f32::MAX, f32::MAX)
        // };
        // let (mut tmaxy, tdeltay) = if f32_not_zero(ray.direction.y) {
        //     (
        //         (next_vox_bound_y - ray.origin.y) / ray.direction.y,
        //         1.0 / ray.direction.y/*  * stepy */
        //     )
        // } else {
        //     (f32::MAX, f32::MAX)
        // };
        // let (mut tmaxz, tdeltaz) = if f32_not_zero(ray.direction.z) {
        //     (
        //         (next_vox_bound_z - ray.origin.z) / ray.direction.z,
        //         1.0 / ray.direction.z/*  * stepz */
        //     )
        // } else {
        //     (f32::MAX, f32::MAX)
        // };
        // let mut diff = (0, 0, 0);
        // let mut neg_ray = false;
        // if coord.x != last_coord.x && ray.direction.x < 0.0 {
        //     diff.0 -= 1;
        //     neg_ray = true;
        // }
        // if coord.y != last_coord.y && ray.direction.y < 0.0 {
        //     diff.1 -= 1;
        //     neg_ray = true;
        // }
        // if coord.z != last_coord.z && ray.direction.z < 0.0 {
        //     diff.2 -= 1;
        //     neg_ray = true;
        // }
        // let mut previous = coord;
        // // if neg_ray {
        // //     coord.x += diff.0;
        // //     coord.y += diff.1;
        // //     coord.z += diff.2;
        // //     let state = self.get_block(coord);
        // //     if filter(self, coord, state) {
        // //         let orientation = state.block().orientation(self, coord, state);
        // //         if state != Id::AIR && state.block().raycast(self, coord, state, orientation) {
        // //             return Some((coord, state));
        // //         }
        // //     }
        // // }
        // let mut iterations = 064;
        // while coord != last_coord {
        //     iterations += 1;
        //     if iterations == 1000 {
        //         return None;
        //     }
        //     if tmaxx < tmaxy {
        //         if tmaxx < tmaxz {
        //             coord.x += stepx as i32;
        //             tmaxx += tdeltax;
        //         } else {
        //             coord.z += stepz as i32;
        //             tmaxz += tdeltaz;
        //         }
        //     } else {
        //         if tmaxy < tmaxz {
        //             coord.y += stepy as i32;
        //             tmaxy += tdeltay;
        //         } else {
        //             coord.z += stepz as i32;
        //             tmaxz += tdeltaz;
        //         }
        //     }
        //     let state = self.get_block(coord);
        //     if filter(self, coord, state) {
        //         let orientation = state.block().orientation(self, coord, state);
        //         if state != Id::AIR && state.block().raycast(self, coord, state, orientation) {
        //             return Some((coord, state));
        //         }
        //     }
        // }
        // None
    }

    pub fn raycast(&self, ray: Ray3d, max_distance: f32) -> Option<RaycastResult> {
        self.raycast_filter(ray, max_distance, |_, _, _| true)
    }

    fn get_region_path(&self, region_x: i32, region_z: i32) -> PathBuf {
        self.subworld_directory.join(format!("{region_x}.{region_z}.rg"))
    }

    pub fn talk_to_bevy(
        &mut self,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<VoxelMaterial>>,
        mut render_chunks: Query<&mut Transform, With<RenderChunkMarker>>,
    ) {
        let mut load = self.load_queue.lend("loading some chunks in talk_to_bevy");
        let start_time = std::time::Instant::now();
        // We'll try 5 milliseconds for now. We only have 16 milliseconds of frame time.
        while start_time.elapsed().as_millis() < 2 {
            if let Some((chunk_x, chunk_z)) = load.pop() {
                let mut chunk = self.chunks.take((chunk_x, chunk_z)).expect("Chunk was not present");
                chunk.load_id.swap_null();
                chunk.unload(self);
                chunk.block_offset = Coord::new(chunk_x * 16, WORLD_BOTTOM, chunk_z * 16);
                let (region_x, region_z) = (chunk_x >> 5, chunk_z >> 5);
                let mut region = if let Some(region) = self.regions.take((region_x, region_z)) {
                    region
                } else {
                    let region_path = self.get_region_path(region_x, region_z);
                    if region_path.is_file() {
                        RegionFile::open_or_create(region_path).expect("Failed to open or create region file.")
                    } else {
                        chunk.edit_time = Timestamp::new(0);
                        chunk.world_gen_id = self.worldgen_queue.insert((chunk_x, chunk_z));
                        self.chunks.set((chunk_x, chunk_z), chunk);
                        continue;
                    }
                };
                let result = self.load_chunk(&mut region, &mut chunk, chunk_x, chunk_z);
                match result {
                    Err(Error::ChunkNotFound) => {
                        chunk.world_gen_id = self.worldgen_queue.insert((chunk_x, chunk_z));
                    }
                    Err(err) => panic!("{err}"),
                    _ => (),
                }
                chunk.edit_time = region.get_timestamp((chunk_x & 31, chunk_z & 31));
                if chunk_x >= self.render_chunks.x_min() &&
                chunk_x < self.render_chunks.x_max() &&
                chunk_z >= self.render_chunks.z_min() &&
                chunk_z < self.render_chunks.z_max() {
                    for y in self.render_chunks.y_min()..self.render_chunks.y_max() {
                        // self.mark_section_dirty(Coord::new(chunk_x, y, chunk_z));
                        let section_index = (y - chunk.section_y()) as usize;
                        let section = &mut chunk.sections[section_index];
                        section.light_dirty.mark();
                        section.blocks_dirty.mark();
                        section.section_dirty.mark();
                        if section.dirty_id.null() {
                            section.dirty_id = self.dirty_queue.insert(Coord::new(chunk_x, y, chunk_z));
                        } else {
                            self.dirty_queue.swap_insert(&mut section.dirty_id, Coord::new(chunk_x, y, chunk_z));
                        }
                    }
                }
                self.regions.set((region_x, region_z), region);
                // if let Some(mut region) = self.regions.take(region_coord) {
                // } else {
                //     // Region was not found, which means the chunk has not been created yet, so generate a new one.
                //     chunk.world_gen_id = self.worldgen_queue.insert((chunk_x, chunk_z));
                // }
                self.chunks.set((chunk_x, chunk_z), chunk);
            } else {
                break;
            }
        }
        self.load_queue.give(load);
        let mut dirty = self.dirty_queue.lend("draining the dirty_queue in talk_to_bevy");
        let start_time = std::time::Instant::now();
        // TODO: Right now, despawning is broken under certain move condition.s
        while start_time.elapsed().as_millis() < 50 {
            if let Some(coord) = dirty.pop() {
                
                let (sect_x, sect_y, sect_z) = coord.into();
                // Let's build the mesh
                let (blocks_dirty, light_map_dirty) = {
                    if let Some(sect) = self.get_section(coord) {
                        (sect.blocks_dirty.dirty(), sect.light_dirty.dirty())
                    } else {
                        (false, false)
                    }
                };
                let make_render_chunk = if let Some(sect) = self.get_section_mut(coord) {
                    sect.blocks_dirty.mark_clean();
                    sect.light_dirty.mark_clean();
                    sect.section_dirty.mark_clean();
                    sect.dirty_id = PoolId::NULL;
                    sect.blocks.is_some()
                } else {
                    panic!("Section not found.");
                };
                // let Some(render_chunk) = self.render_chunks.get_opt_mut(coord) else {
                //     panic!("Render chunk out of bounds");
                // };
                let mut render_chunk = self.render_chunks.take(coord);
                if make_render_chunk {
                    if render_chunk.is_none() {
                        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
                        MeshBuilder::build_mesh(&mut mesh, |build| ());
                        let mesh = meshes.add(mesh);
                        let material = materials.add(VoxelMaterial::new(self.array_texture.clone()));
                        let (x, y, z) = (
                            (coord.x * 16) as f32,
                            (coord.y * 16) as f32,
                            (coord.z * 16) as f32
                        );
                        use bevy::render::primitives::Aabb;
                        let aabb = Aabb::from_min_max(vec3(0.0, 0.0, 0.0), vec3(16.0, 16.0, 16.0));
                        let entity = commands.spawn((
                            MaterialMeshBundle {
                                mesh: mesh.clone(),
                                transform: Transform::from_xyz(x, y, z),
                                material: material.clone(),
                                ..Default::default()
                            },
                            aabb,
                            RenderChunkMarker
                        )).id();
                        render_chunk.replace(RenderChunk {
                            mesh: mesh,
                            material: material,
                            move_id: PoolId::NULL,
                            entity,
                        });
                        // Some()
                        // let render_chunk = self.render_chunks.get_mut(coord).expect("Failed to get render chunk");
                    }
                } else {
                    if let Some(unload_chunk) = render_chunk.take() {
                        self.move_render_chunk_queue.remove(unload_chunk.move_id);
                        commands.entity(unload_chunk.entity).despawn_recursive();
                    }
                }
                let Some(render_chunk_mut) = render_chunk.as_mut() else {
                    self.render_chunks.set_opt(coord, render_chunk);
                    continue;
                };
                if blocks_dirty {
                    let mesh = meshes.get_mut(render_chunk_mut.mesh.id()).expect("Failed to get the mesh");
                    MeshBuilder::build_mesh(mesh, |build| {
                        for y in 0..16 {
                            for z in 0..16 {
                                'xloop: for x in 0..16 {
                                    let block_coord = (sect_x * 16 + x, sect_y * 16 + y, sect_z * 16 + z);
                                    let offset = vec3(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                                    let state = self.get_block(block_coord);
                                    if state == Id::AIR {
                                        continue 'xloop;
                                    }
                                    let orientation = state.block().orientation(self, block_coord.into(), state);
                                    let occlusion = self.get_occlusion(block_coord);
                                    build.set_offset(offset);
                                    build.set_orientation(orientation);
                                    state.block().push_mesh(build, self, block_coord.into(), state, occlusion, orientation);
                                }
                            }
                        }
                    });
                }
                if light_map_dirty {
                    // TODO: Rebuild lightmap
                }
                self.render_chunks.set_opt(coord, render_chunk);
            } else {
                break;
            }
        }
        self.dirty_queue.give(dirty);
        let mut move_render_chunk_queue = self.move_render_chunk_queue.lend("Moving chunk entities");
        move_render_chunk_queue.drain().for_each(|coord| {
            // We expect that if a render chunk requested to move, that means that it's not None.
            let rend_chunk = self.render_chunks.get_mut(coord).expect("Render chunk was not found");
            let ent = rend_chunk.entity.clone();
            rend_chunk.move_id = PoolId::NULL;
            let mut trans = render_chunks.get_mut(ent).expect("Failed to get transform");
            let offset = vec3((coord.x * 16) as f32, (coord.y * 16) as f32, (coord.z * 16) as f32);
            trans.translation = offset;
        });
        self.move_render_chunk_queue.give(move_render_chunk_queue);
    }

    pub fn load_chunk(&mut self, region: &mut RegionFile, chunk: &mut Chunk, x: i32, z: i32) -> crate::core::error::Result<()> {
        // self.unload_chunk(chunk);
        chunk.block_offset = Coord::new(x * 16, WORLD_BOTTOM, z * 16);
        region.read((x & 31, z & 31), |reader| {
            chunk.read_from(reader, self)
        })
        // match result {
        //     Err(Error::ChunkNotFound) => {
        //         // If the chunk was not found, generate a new one.
        //         chunk.world_gen_id = self.worldgen_queue.insert((x, z));
        //     },
        //     err => err?,
        //     Ok(_) => (),
        // }
        // Ok(())
    }

    pub fn move_center<C: Into<(i32, i32, i32)>>(&mut self, center: C) {
        let center: (i32, i32, i32) = center.into();
        let center: Coord = center.into();
        let padded_distance = self.render_distance + WORLD_SIZE_PAD as i32;
        let padded_size = padded_distance * 2;
        let render_min = self.render_chunks.bounds().min;
        let (render_x, render_y, render_z) = calculate_center_offset(self.render_distance, center, Some(Self::WORLD_BOUNDS)).section_coord().xyz();
        let (chunk_x, chunk_z) = calculate_center_offset(padded_distance, center, Some(Self::WORLD_BOUNDS)).chunk_coord().xz();
        let (region_x, region_z) = calculate_region_min((chunk_x, chunk_z));
        // World hasn't moved
        if render_min == (render_x, render_y, render_z) {
            return;
        }
        let chunk_min = self.chunks.bounds().min;
        // Chunks moved
        if chunk_min != (chunk_x, chunk_z) {
            // This operation will be kinda slow if a lot of chunks need to be saved.
            // Thankfully that shouldn't be too much of a problem since you can expect that only the nearest 4 chunks might be edited before the world moves.
            self.save_world().expect("Failed to save the world");
        }
        // take temporary ownership of 
        let mut regions = self.regions.lend("regions in move_center");
        let result = regions.try_reposition((region_x, region_z), |old_pos, (x, z), region| {
            let rg_path = self.get_region_path(x, z);
            if rg_path.is_file() {
                Result::Ok(Some(RegionFile::open(rg_path)?))
            } else {
                // There's no region file, so just return None. We're not reusing RegionFile instances.
                Ok(None)
            }
        }).handle_err(|err| {
            panic!("Error from regions.try_reposition: {err}");
        });
        let mut chunks = self.chunks.lend("chunks in move_center");
        chunks.reposition((chunk_x, chunk_z), |old_pos, (x, z), chunk| {
            //                   The chunk should never be None. If it is, that's an error.
            let mut chunk = chunk.expect("Chunk was None");
            if chunk.load_id.null() {
                chunk.load_id = self.load_queue.insert((x, z));
            } else {
                self.load_queue.swap_insert(&mut chunk.load_id, (x, z));
            }
            // self.unload_chunk(&mut chunk);
            // chunk.block_offset = Coord::new(x * 16, WORLD_BOTTOM, z * 16);
            // if let Some(region) = regions.get_mut((x >> 5, z >> 5)) {
            //     let result = region.read((x & 31, z & 31), |reader| {
            //         chunk.read_from(reader, self)
            //     });
            //     match result {
            //         Err(Error::ChunkNotFound) => {
                        
            //         },
            //         Err(err) => {
            //             panic!("Error: {err}");
            //         }
            //         _ => (),
            //     }
            //     chunk.edit_time = region.get_timestamp((x & 31, z & 31));
            // }
            // chunk.block_offset.x = x * 16;
            // chunk.block_offset.z = z * 16;
            Some(chunk)
        });
        self.chunks.give(chunks);
        self.regions.give(regions);
        let mut render_chunks = self.render_chunks.lend("render_chunks in move_center");
        render_chunks.reposition((render_x, render_y, render_z), |old_pos, new_pos, mut chunk| {
            if let Some(rendchunk) = &mut chunk {
                let old_id = rendchunk.move_id.swap(PoolId::NULL);
                if !old_id.null() {
                    self.move_render_chunk_queue.remove(old_id);
                }
                rendchunk.move_id = self.move_render_chunk_queue.insert(Coord::from(new_pos));
            }
            let section_coord: Coord = new_pos.into();
            let sect = self.get_section_mut(new_pos.into()).expect("failed to get section in render_chunks.reposition");
            sect.light_dirty.mark();
            sect.blocks_dirty.mark();
            sect.section_dirty.mark();
            let Some(mut block_chunk) = self.chunks.get_mut(section_coord.xz()) else {
                panic!("Chunk was None");
            };
            let section_index = (section_coord.y - block_chunk.section_y()) as usize;
            let dirty_id = block_chunk.sections[section_index].dirty_id.swap_null();
            if dirty_id.non_null() {
                // Gotta remove the old one
                self.dirty_queue.remove(dirty_id);
            }
            block_chunk.sections[section_index].dirty_id = self.dirty_queue.insert(section_coord);
            chunk
        });
        self.render_chunks.give(render_chunks);

    }

    #[must_use]
    pub fn save_world(&mut self) -> Result<()> {
        self.save_queue.drain().try_for_each(|coord| {
            let (chunk_x, chunk_z) = coord.xz();
            let mut chunks = self.chunks.lend("chunks in save_world");
            let mut regions = self.regions.lend("regions in save_world");
            let mut chunk = chunks.take((chunk_x, chunk_z)).expect("Chunk was None");
            // chunk.save_id = PoolId::NULL;
            let (region_x, region_z) = (chunk_x >> 5, chunk_z >> 5);
            let mut region = if let Some(region) = regions.take((region_x, region_z)) {
                region
            } else {
                RegionFile::open_or_create(self.subworld_directory.join(format!("{region_x}.{region_z}.rg")))?
            };
            let result = region.write_timestamped((chunk_x, chunk_z), chunk.edit_time, |writer| {
                chunk.write_to(writer)?;
                Ok(())
            });
            match result {
                Err(err) => {
                    panic!("{err} at {chunk_x} {chunk_z}");
                },
                _ => (),
            }
            chunk.save_id = PoolId::NULL;
            chunks.set((chunk_x, chunk_z), chunk);
            regions.set((region_x, region_z), region);
            self.chunks.give(chunks);
            self.regions.give(regions);
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
        chunk.unload(self);
    }

    pub fn mark_section_dirty(&mut self, section_coord: Coord) {
        let block_y = section_coord.y * 16;
        if !self.render_chunks.bounds().contains(section_coord)
        || block_y < WORLD_BOTTOM
        || block_y >= WORLD_TOP {
            return;
        }
        let Some(mut chunk) = self.chunks.get_mut(section_coord.xz()) else {
            panic!("Chunk was None");
        };
        let section_index = (section_coord.y - chunk.section_y()) as usize;
        if chunk.sections[section_index].dirty_id.null() {
            chunk.sections[section_index].dirty_id = self.dirty_queue.insert(section_coord);
        }
        
        // self.chunks.set(section_coord.xz(), chunk);
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

    
    pub fn get_section(&self, section_coord: Coord) -> Option<&Section> {
        let chunk = self.chunks.get((section_coord.x, section_coord.z))?;
        let y = section_coord.y - (chunk.block_offset.y >> 4);
        if y < 0 || y as usize >= chunk.sections.len() {
            return None;
        }
        Some(&chunk.sections[y as usize])
    }

    
    pub fn get_section_mut(&mut self, section_coord: Coord) -> Option<&mut Section> {
        let chunk = self.chunks.get_mut((section_coord.x, section_coord.z))?;
        let y = section_coord.y - (chunk.block_offset.y >> 4);
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

    pub fn query<'a, C: Into<(i32, i32, i32)>, T: VoxelQuery<'a>>(&'a self, coord: C) -> T::Output {
        let coord: (i32, i32, i32) = coord.into();
        let coord: Coord = coord.into();
        if !self.bounds().contains(coord) {
            return T::default();
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.query::<T>(coord)
    }

    fn set_update_ref(&mut self, coord: Coord, value: UpdateRef) -> UpdateRef {
        if !self.bounds().contains(coord) {
            return UpdateRef::NULL;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get_mut((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.set_update_ref(coord, value)
    }

    fn get_update_ref(&self, coord: Coord) -> UpdateRef {
        if !self.bounds().contains(coord) {
            return UpdateRef::NULL;
        }
        let chunk_x = coord.x >> 4;
        let chunk_z = coord.z >> 4;
        let chunk = self.chunks.get((chunk_x, chunk_z)).expect("Chunk was None");
        chunk.get_update_ref(coord)
    }

    /// This will get the block [Id] at the given coordinate if the coordinate is not out of bounds.
    /// If the coordinate is out of bounds, it will return [Id::AIR].
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
        if !self.bounds().contains(coord) {
            return Id::AIR;
        }
        let old = self.get_block(coord);
        if state == old {
            return old;
        }
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
        let (state, enable) = (place_context.replacement, place_context.enable);
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
                self.mark_modified(coord.chunk_coord());
                if self.render_bounds().contains(coord) {
                    let section_coord = coord.section_coord();
                    // self.dirty_sections.push(section_coord);
                    self.mark_section_dirty(section_coord);
                }
                let block = state.block();
                let my_layer = block.layer(self, coord, state);
                let my_orient = block.orientation(self, coord, state);
                let my_occl = block.occluder(self, state);
                let my_occlee = block.occludee(self, state);
                let neighbors = self.neighbors(coord);
                Direction::iter().for_each(|dir| {
                    let adj_dir = dir.invert();
                    let adj_state = neighbors[dir];
                    let adj_block = adj_state.block();
                    let adj_coord = coord + dir;
                    let adj_layer = adj_block.layer(self, adj_coord, adj_state);
                    if adj_state != Id::AIR {
                        adj_block.neighbor_updated(self, adj_dir, adj_coord, coord, adj_state, state);
                    }
                    // No occlusion happens if they are on different layers.
                    if adj_layer != my_layer {
                        self.show_face(coord, dir);
                        self.show_face(adj_coord, adj_dir);
                        return;
                    }
                    let adj_orient = adj_block.orientation(self, adj_coord, adj_state);
                    let adj_occl = adj_block.occluder(self, adj_state);
                    let adj_occlee = adj_block.occludee(self, adj_state);
                    if my_occlee.occluded_by(my_orient, dir, adj_occl, adj_orient) {
                        // println!("My Hide face {coord} {dir}");
                        self.hide_face(coord, dir);
                    } else {
                        // println!("My Show face {coord} {dir}");
                        self.show_face(coord, dir);
                    }
                    if adj_occlee.occluded_by(adj_orient, adj_dir, my_occl, my_orient) {
                        // println!("Adj Hide face {adj_coord} {dir}");
                        self.hide_face(adj_coord, adj_dir);
                    } else {
                        // println!("Adj Show face {adj_coord} {dir}");
                        self.show_face(adj_coord, adj_dir);
                    }
                });
                
                old
            },
        }
    }

    pub fn get_occlusion<C: Into<(i32, i32, i32)>>(&self, coord: C) -> Occlusion {
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
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                let sect = self.get_section_mut(section_coord).unwrap();
                sect.blocks_dirty.mark();
                sect.section_dirty.mark();
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
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
                let sect = self.get_section_mut(section_coord).unwrap();
                sect.blocks_dirty.mark();
                sect.section_dirty.mark();
                self.mark_section_dirty(section_coord);
            }
        }
        change.change
    }

    pub fn get_light_level<C: Into<(i32, i32, i32)>>(&self, coord: C) -> u8 {
        let (block_light, sky_light) = self.query::<C, (BlockLight, SkyLight)>(coord);
        block_light.max(sky_light)
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
            if self.render_bounds().contains(coord) {
                let section_coord = coord.section_coord();
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
        if !self.bounds().contains(coord) {
            return false;
        }
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
        if !self.bounds().contains(coord) {
            return false;
        }
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
        if let Some(chunk) = self.chunks.get((chunk_x, chunk_z)) {
            chunk.height(x, z)
        } else {
            WORLD_BOTTOM
        }
    }

    pub fn neighbors<C: Into<(i32, i32, i32)>>(&self, coord: C) -> Faces<Id> {
        cast_coord!(coord);
        use Direction::*;
        macro_rules! get_faces {
            ($($dir:expr),*) => {
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
            NegX,
            NegY,
            NegZ,
            PosX,
            PosY,
            PosZ
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
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
    pub material: Handle<VoxelMaterial>,
    pub move_id: PoolId<MoveRenderChunkMarker>,
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