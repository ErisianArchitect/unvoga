use std::{sync::Arc, thread};

use rollgrid::rollgrid3d::Bounds3D;

use crate::{blockstate, core::{math::coordmap::Rotation, voxel::{block::Block, blocks::{self, StateRef}, blockstate::StateValue, coord::Coord, direction::Direction, faces::Faces, occluder::Occluder, occlusion_shape::{OcclusionShape, OcclusionShape16x16, OcclusionShape2x2}, tag::Tag, world::VoxelWorld}}};

pub fn sandbox() {
    use crate::core::voxel::direction::Direction;

    println!("World Test");
    let mut world = VoxelWorld::new(16, Coord::new(0, 0, 0));
    let usage = world.dynamic_usage();
    println!("Memory Usage: {usage}");
    blocks::register_block(DirtBlock);
    blocks::register_block(RotatedBlock);
    blocks::register_block(DebugBlock);
    println!(" World Bounds: {:?}", world.bounds());
    println!("Render Bounds: {:?}", world.render_bounds());
    println!("  Block Count: {}", world.bounds().volume());
    let air = StateRef::AIR;
    let debug = blockstate!(debug).register();
    let debug_data = blockstate!(debug, withdata = true).register();
    let enabled = blockstate!(debug, enabled = true).register();
    let dirt = blockstate!(dirt).register();
    let rot1 = blockstate!(rotated, rotation=Rotation::new(Direction::PosY, 0)).register();
    let rot2 = blockstate!(rotated, rotation=Rotation::new(Direction::PosZ, 3)).register();

    for i in 0..10 {
        world.set_block((i, 0, 0), enabled);
    }
    world.update();
    let data = world.take_data((0, 0, 0));
    println!("{data:?}");
    let result = world.message((0, 0, 0), "Hello, from sandbox()");
    println!("{result:?}");
    // world.set((0, 1, 0), rot2);
    
    // let c = (1, 1, 1);
    // world.set(c, debug_data);
    let now = std::time::Instant::now();
    let bounds = Bounds3D::new(
        (0, -272, 0),
        (128, 240, 128)
    );
    // bounds.iter().for_each(|coord| {
    //     world.set(coord, debug_data);
    //     world.set_block_light(coord, 1);
    //     world.set_sky_light(coord, 2);
    //     // world.set_data(coord, Tag::Bool(true));
    // });
    let elapsed = now.elapsed();
    println!("Set {} blocks in world bounds in {:.3} seconds.", bounds.volume(), elapsed.as_secs_f64());
    let usage = world.dynamic_usage();
    println!("Memory Usage: {usage}");
    // itertools::iproduct!(-16..32, -16..32, -16..32).map(|(y, z, x)| (x, y, z))
    // .for_each(|coord| {
    //     let (x, y, z) = coord;
    //     world.delete_data(coord);
    //     world.set(coord, air);
    //     world.set_sky_light(coord, 0);
    //     world.set_block_light(coord, 0);
    // });
    // world.set_sky_light(c, 1);
    // world.set_block_light(c, 1);
    // world.set_sky_light(c, 0);
    // world.set_block_light(c, 0);
    // world.set_block_light(c, 1);
    // world.set_sky_light(c, 1);

    // itertools::iproduct!(15..16, 0..1, 15..16).for_each(|(y, z, x)| {
    // itertools::iproduct!(0..16, 0..16, 0..16).for_each(|(y, z, x)| {
    //     world.set((x, y, z), debug_data);
    //     world.set_block_light((x, y, z), 8);
    //     world.set_sky_light((x, y, z), 15);
    //     world.set_data((x, y, z), Tag::Null);
    // });
    // world.set((0, 0, 0), debug_data);
    // world.set(( 0, 15, 15), debug_data);
    // world.set((15, 15, 15), debug_data);

    // itertools::iproduct!(0..16, 0..16, 0..16).for_each(|(y, z, x)| {
    //     world.set_block_light((x, y, z), 0);
    //     world.set_sky_light((x, y, z), 0);
    //     world.set((x, y, z), air);
    // });
    // itertools::iproduct!(0..16, 0..16, 0..16).map(|(y, z, x)| (x, y, z)).for_each(|(x, y, z)| {
    //     let faces = world.occlusion((x, y, z));
    //     if faces != Occlusion::UNOCCLUDED {
    //         println!("Occluded at ({x:2}, {y:2}, {z:2}) {faces}");
    //     }
    // });
    let usage = world.dynamic_usage();
    println!("Memory: {usage}");
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
        const OCCLUDER: Occluder = Occluder {
            neg_x: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
                [1, 0],
                [1, 1],
            ])),
            neg_y: OcclusionShape::Full,
            neg_z: OcclusionShape::Full,
            pos_x: OcclusionShape::Full,
            pos_y: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
                [1, 0],
                [1, 1],
            ])),
            pos_z: OcclusionShape::Full,
        };
        &OCCLUDER
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
    fn occluder(&self, state: StateRef) -> &Occluder {
        const OCCLUDER: Occluder = Occluder {
            neg_x: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
                [1, 0],
                [1, 1],
            ])),
            neg_y: OcclusionShape::Full,
            neg_z: OcclusionShape::Full,
            pos_x: OcclusionShape::Full,
            pos_y: OcclusionShape::S2x2(OcclusionShape2x2::from_matrix([
                [1, 0],
                [1, 1],
            ])),
            pos_z: OcclusionShape::Full,
        };
        &OCCLUDER
    }
    fn default_state(&self) -> crate::core::voxel::blockstate::BlockState {
        blockstate!(debug)
    }
    fn message(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, message: Tag) -> Tag {
        println!("Message received: {message:?}");
        Tag::from("Debug Message Result")
    }
    fn on_place(&self, world: &mut VoxelWorld, coord: Coord, old: StateRef, new: StateRef) {
        // println!("On Place {coord} old = {old} new = {new}");
        if matches!(new["withdata"], StateValue::Bool(true)) {
            // println!("Adding data...");
            world.set_data(coord, Tag::from("The quick brown fox jumps over the lazy dog."));
        }
        if matches!(new["enabled"], StateValue::Bool(true)) {
            world.enable(coord);
        }
    }
    fn on_remove(&self, world: &mut VoxelWorld, coord: Coord, old: StateRef, new: StateRef) {
        // println!("On Remove {coord} old = {old} new = {new}");
    }
    fn data_set(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, data: &mut Tag) {
        // println!("Data Set {coord} state = {state} data = {data:?}");
    }
    fn data_deleted(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, data: Tag) {
        // println!("Data Deleted {coord} state = {state} data = {data:?}");
    }
    fn light_updated(&self, world: &mut VoxelWorld, coord: Coord, old_level: u8, new_level: u8) {
        // println!("Light Updated {coord} old = {old_level} new = {new_level}");
    }
    fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: StateRef, neighbor_state: StateRef) {
        // println!("Neighbor Updated {coord} -> {neighbor_coord} {state} -> {neighbor_state}");
    }
    fn update(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef) {
        println!("Update {coord} {state}");
    }
}