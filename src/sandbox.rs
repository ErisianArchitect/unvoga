use std::{sync::Arc, thread, time::Duration};

use rollgrid::rollgrid3d::Bounds3D;

use crate::{blockstate, core::{math::coordmap::Rotation, util::counter::AtomicCounter, voxel::{block::Block, blocks::{self, Id}, blockstate::StateValue, coord::Coord, direction::Direction, faces::Faces, occluder::Occluder, occlusion_shape::{OcclusionShape, OcclusionShape16x16, OcclusionShape2x2}, tag::Tag, world::{query::Enabled, PlaceContext, VoxelWorld}}}};

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
    let air = Id::AIR;
    let debug = blockstate!(debug).register();
    let debug_data = blockstate!(debug, withdata = true).register();
    let enabled = blockstate!(debug, enabled = true).register();
    let dirt = blockstate!(dirt).register();
    let rot1 = blockstate!(rotated, rotation=Rotation::new(Direction::PosY, 0)).register();
    let rot2 = blockstate!(rotated, rotation=Rotation::new(Direction::PosZ, 3)).register();

    itertools::iproduct!(0..2, 0..2).for_each(|(y, x)| {
        world.set_block((x, 0, y), enabled);
    });
    world.set_block((13,12, 69), debug_data);
    world.set_enabled((13,12,69), true);
    let (state, enabled): (Id, bool) = world.query::<_, (Id, Enabled)>((13,12,69));
    println!("{state} {enabled}");
    println!("Frame 1");
    world.update();
    world.set_enabled((13,12,69), false);
    println!("Frame 2");
    world.update();
    
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
            context: &mut PlaceContext,
        ) {
            // world.set_block(coord, Id::AIR);
            println!("dirt placed: {}", context.replacement());
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

    fn occluder(&self, world: &VoxelWorld, state: Id) -> &Occluder {
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

    fn rotation(&self, world: &VoxelWorld, coord: Coord, state: Id) -> Rotation {
        if let Some(&StateValue::Rotation(rotation)) = state.get_property("rotation") {
            rotation
        } else {
            Rotation::default()
        }
    }
    fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: Id, neighbor_state: Id) {
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
    fn occluder(&self, world: &VoxelWorld, state: Id) -> &Occluder {
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
    fn call(&self, world: &mut VoxelWorld, coord: Coord, state: Id, function: &str, arg: Tag) -> Tag {
        println!("Message received: {arg:?}");
        Tag::from("Debug Message Result")
    }
    fn on_place(&self, world: &mut VoxelWorld, context: &mut PlaceContext) {
        let (coord, old, new) = (
            context.coord(),
            context.old(),
            context.replacement()
        );
        println!("On Place {coord} old = {old} new = {new}");
        if matches!(context.replacement()["replace"], StateValue::Bool(true)) {
            context.replace(blockstate!(debug).register());
        } else if matches!(context.replacement()["withdata"], StateValue::Bool(true)) {
            println!("Adding data...");
            world.set_data(context.coord(), Tag::from("The quick brown fox jumps over the lazy dog."));
        }
    }
    fn on_remove(&self, world: &mut VoxelWorld, coord: Coord, old: Id, new: Id) {
        println!("On Remove {coord} old = {old} new = {new}");
    }
    fn on_data_set(&self, world: &mut VoxelWorld, coord: Coord, state: Id, data: &mut Tag) {
        println!("Data Set {coord} state = {state} data = {data:?}");
    }
    fn on_data_delete(&self, world: &mut VoxelWorld, coord: Coord, state: Id, data: Tag) {
        println!("Data Deleted {coord} state = {state} data = {data:?}");
    }
    fn light_updated(&self, world: &mut VoxelWorld, coord: Coord, old_level: u8, new_level: u8) {
        println!("Light Updated {coord} old = {old_level} new = {new_level}");
    }
    fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: Id, neighbor_state: Id) {
        println!("Neighbor Updated {coord} -> {neighbor_coord} {state} -> {neighbor_state}");
    }
    fn on_update(&self, world: &mut VoxelWorld, coord: Coord, state: Id) {
        println!("Update {coord} {state}");
        // world.set_block(coord + Direction::PosY, state);
    }
    fn default_enabled(&self, world: &VoxelWorld, coord: Coord, state: Id) -> bool {
        matches!(state["enabled"], StateValue::Bool(true))
    }
}