use std::any::Any;
use crate::core::math::coordmap::Rotation;

use super::{blocks::StateRef, blockstate::BlockState, coord::Coord, direction::Direction, engine::VoxelEngine, faces::Faces, lighting::lightargs::LightArgs, occluder::Occluder, occlusion_shape::OcclusionShape, tag::Tag, world::{occlusion::Occlusion, PlaceContext, VoxelWorld}};

use crate::prelude::Rgb;

pub trait Block: Any {
    fn name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn occluder(&self, world: &VoxelWorld, state: StateRef) -> &Occluder {
        const FULL_FACES: Occluder = Occluder::new(
            OcclusionShape::Full,
            OcclusionShape::Full,
            OcclusionShape::Full,
            OcclusionShape::Full,
            OcclusionShape::Full,
            OcclusionShape::Full,
        );
        &FULL_FACES
    }
    fn material(&self, world: &VoxelWorld, coord: Coord, state: StateRef, face: Direction) -> () {
        todo!()
    }
    fn color(&self, world: &VoxelWorld, coord: Coord, state: StateRef, face: Direction) -> Rgb {
        Rgb::new(255, 0, 255)
    }
    fn rotation(&self, world: &VoxelWorld, coord: Coord, state: StateRef) -> Rotation {
        Rotation::new(Direction::PosY, 0)
    }
    fn default_enabled(&self, world: &VoxelWorld, coord: Coord, state: StateRef) -> bool { false }
    fn light_args(&self, world: &VoxelWorld, coord: Coord, state: StateRef) -> LightArgs {
        LightArgs::new(15, 0)
    }
    fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: StateRef, neighbor_state: StateRef) {}
    fn light_updated(&self, world: &mut VoxelWorld, coord: Coord, old_level: u8, new_level: u8) {}
    fn call(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, function: &str, arg: Tag) -> Tag { Tag::Null }
    fn on_entity_collide(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, face: Direction, entity: ()) {}
    fn on_interact(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef) {}
    fn on_update(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef) {}
    fn on_place(&self, world: &mut VoxelWorld, context: &mut PlaceContext) { }
    fn on_remove(&self, world: &mut VoxelWorld, coord: Coord, old: StateRef, new: StateRef) {}
    fn on_data_set(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, data: &mut Tag) {}
    fn on_data_delete(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, data: Tag) {}
    fn on_enabled_changed(&self, world: &mut VoxelWorld, coord: Coord, state: StateRef, enabled: bool) {}
    fn push_mesh(&self, mesh_builder: &mut (), coord: Coord, state: StateRef, occlusion: Occlusion, rotation: Rotation) {}
    fn rotate(&self, coord: Coord, state: StateRef, rotation: Rotation) -> StateRef { state }
    fn default_state(&self) -> BlockState;

}

#[test]
fn borrow_test() {
    struct Foo(String);

    fn foo<S: AsRef<str>>(s: S) -> Foo {
        Foo::new(s)
    }

    impl Foo {
        fn new<S: AsRef<str>>(s: S) -> Self {
            Self(s.as_ref().to_owned())
        }

        fn bar(&self, world: &mut World, baz: i32) {
            // todo
        }
    }

    struct World {
    }

    struct Engine {
        foos: Vec<Foo>,
        world: World,
    }

    let mut engine = Engine {
        foos: vec![foo("foo"), foo("bar"), foo("baz")],
        world: World {

        }
    };
    engine.foos[1].bar(&mut engine.world, 3);
}