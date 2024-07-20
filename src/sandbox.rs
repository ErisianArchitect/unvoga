use std::path::PathBuf;
use std::{sync::Arc, thread, time::Duration};

use bevy::math::IVec2;
use hashbrown::HashMap;
use rollgrid::rollgrid3d::Bounds3D;

use crate::core::voxel::region::regionfile::RegionFile;
use crate::prelude::*;
use crate::core::error::*;
use crate::{blockstate, core::{math::coordmap::Rotation, util::counter::AtomicCounter, voxel::{block::Block, blocks::{self, Id}, blockstate::StateValue, coord::Coord, direction::Direction, faces::Faces, occluder::Occluder, occlusion_shape::{OcclusionShape, OcclusionShape16x16, OcclusionShape2x2}, tag::Tag, world::{query::Enabled, PlaceContext, VoxelWorld}}}};

pub fn sandbox() {

    use crate::core::voxel::direction::Direction;

    println!("World Test");
    blocks::register_block(DirtBlock);
    blocks::register_block(RotatedBlock);
    blocks::register_block(DebugBlock);
    let mut world = VoxelWorld::new(16, Coord::new(0, 0, 0), "ignore/test_world");
    let usage = world.dynamic_usage();
    println!("Memory Usage: {usage}");
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

    println!("Getting block");
    let block = world.get_block((0, 0, 0));
    println!("{block}");
    world.set_block((0, 0, 0), debug_data);
    world.save_world();
    // let coord = Coord::new(13,12,69).chunk_coord();
    // {
    //     let chunk = world.get_chunk((coord.x, coord.z)).unwrap();
    //     println!("Edit Time: {}", chunk.edit_time.0);
    // }
    // std::thread::sleep(Duration::from_secs(2));
    // itertools::iproduct!(0..2, 0..2).for_each(|(y, x)| {
    //     world.set_block((x, 0, y), enabled);
    // });
    // world.set_block((13,12, 69), debug_data);
    // world.call((13,12,69), "test", Tag::from("Hello, world"));
    // world.call((13,12,69), "set_enabled", true);
    // let (state, enabled): (Id, bool) = world.query::<_, (Id, Enabled)>((13,12,69));
    // println!("{state} {enabled}");
    // println!("Frame 1");
    // {
    //     let chunk = world.get_chunk((coord.x, coord.z)).unwrap();
    //     println!("Edit Time: {}", chunk.edit_time.0);
    // }
    // world.update();
    // world.call((13,12,69), "set_enabled", false);
    // println!("Frame 2");
    // world.update();
    

    // println!("Write/Read Test");
    // let now = std::time::Instant::now();
    // write_read_test().expect("Failure");
    // let elapsed = now.elapsed();
    // println!("Elapsed secs: {}", elapsed.as_secs_f64());
    
    // let usage = world.dynamic_usage();
    // println!("Memory: {usage}");
}

#[test]
fn bitmask_test() {
    let mask = i8::bitmask_range(1..7);
    println!("{mask:08b}");
}

fn write_read_test() -> Result<()> {
    let path: PathBuf = "ignore/test.rg".into();
    use rand::prelude::*;
    use rand::rngs::OsRng;
    let mut seed = [0u8; 32];
    OsRng.fill_bytes(&mut seed);
    let mut rng = StdRng::from_seed(seed);
    {
        let mut region = RegionFile::create(&path)?;
        for z in 0..32 {
            for x in 0..32 {
                let array = Tag::from(Array::U8((0u32..4096*511+1234).map(|i| rng.gen()).collect()));
                let position = Tag::IVec2(IVec2::new(x as i32, z as i32));
                let tag = Tag::from(HashMap::from([
                    ("array".to_owned(), array.clone()),
                    ("position".to_owned(), position.clone())
                ]));
                region.write_value((x, z), &tag)?;
            }
        }
    }
    let mut rng = StdRng::from_seed(seed);
    {
        let mut region = RegionFile::open(&path)?;
        for z in 0..32 {
            for x in 0..32 {
                let array = Box::new(Array::U8((0u32..4096*511+1234).map(|i| rng.gen()).collect()));
                let position = IVec2::new(x as i32, z as i32);
                let read_tag: Tag = region.read_value((x, z))?;
                
                if let (
                    Tag::Array(read_array),
                    Tag::IVec2(read_position)
                ) = (&read_tag["array"], &read_tag["position"]) {
                    assert_eq!(&array, read_array);
                    assert_eq!(&position, read_position);
                } else {
                    panic!("Tag not read.")
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod testing_sandbox {
    use std::sync::OnceLock;

    #[derive(Debug)]
    struct OnDrop(u32);

    impl OnDrop {
        fn reset(&mut self) {
            self.0 = 0;
        }
    }

    impl Drop for OnDrop {
        fn drop(&mut self) {
            println!("Dropping id: {}", self.0);
        }
    }

    use super::*;
    #[test]
    fn sandbox() {
        static mut DATA: OnceLock<Vec<OnDrop>> = OnceLock::new();
        unsafe {
            DATA.set(Vec::new());
            let Some(data) = DATA.get_mut() else {
                panic!();
            };
            data.push(OnDrop(0));
            data.push(OnDrop(1));
            data.push(OnDrop(2));
            println!("Before First Removal");
            data[1] = OnDrop(5);
            data[2].reset();
            data[2] = OnDrop(0);
            println!("After First Removal");
            DATA.take();
            println!("After Take");
        }

    }
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
        match function {
            "test" => println!("test({arg:?})"),
            "disable" => {
                println!("Disabling.");
                world.disable(coord);
            }
            "set_enabled" => match arg {
                Tag::Bool(true) => world.enable(coord),
                Tag::Bool(false) => world.disable(coord),
                _ => println!("Invalid argument."),
            }
            other => println!("{other}({arg:?})"),
        }
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
    fn enable_on_place(&self, world: &VoxelWorld, coord: Coord, state: Id) -> bool {
        matches!(state["enabled"], StateValue::Bool(true))
    }
}