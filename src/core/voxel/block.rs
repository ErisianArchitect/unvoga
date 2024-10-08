#![allow(unused)]
use std::any::Any;
use bevy::math::Ray3d;

use crate::{core::math::aabb::AABB, prelude::*};

use super::{blocklayer::BlockLayer, blocks::Id, blockstate::BlockState, coord::Coord, direction::Direction, engine::VoxelEngine, faces::Faces, level_of_detail::LOD, lighting::lightargs::LightArgs, occluder::Occluder, occlusionshape::OcclusionShape, rendering::meshbuilder::MeshBuilder, tag::Tag, world::{occlusion::Occlusion, PlaceContext, VoxelWorld}};

use crate::prelude::Rgb;

mod sealed {
    use super::Block;

    pub trait Sealed {}

    impl<B: Block> Sealed for B {}
}

pub trait Block: Any {
    fn name(&self) -> &str;
    fn on_register(&mut self) {}
    /// The occluder that occludes
    fn occluder(&self, world: &VoxelWorld, state: Id) -> &Occluder {
        &Occluder::FULL_FACES
    }
    /// The occluder that is occluded
    fn occludee(&self, world: &VoxelWorld, state: Id) -> &Occluder {
        &Occluder::FULL_FACES
    }
    fn material(&self, world: &VoxelWorld, coord: Coord, state: Id, face: Direction) -> () {
        todo!()
    }
    fn color(&self, world: &VoxelWorld, coord: Coord, state: Id, face: Direction) -> Rgb {
        Rgb::new(255, 0, 255)
    }
    fn layer(&self, world: &VoxelWorld, coord: Coord, state: Id) -> BlockLayer {
        BlockLayer::Base
    }
    fn orientation(&self, world: &VoxelWorld, coord: Coord, state: Id) -> Orientation {
        Orientation::default()
    }
    /// This handles returning a new state when a state is oriented.
    /// (Use [Orientation::reorient] in most cases where you want to reorient a block)
    /// Return the given state if you want the state to remain unchanged.
    fn reorient(&self, world: &VoxelWorld, coord: Coord, state: Id, orientation: Orientation) -> Id {
        state
    }
    fn enable_on_place(&self, world: &VoxelWorld, coord: Coord, state: Id) -> bool { false }
    fn light_args(&self, world: &VoxelWorld, coord: Coord, state: Id) -> LightArgs {
        LightArgs::new(15, 0)
    }
    fn neighbor_updated(&self, world: &mut VoxelWorld, direction: Direction, coord: Coord, neighbor_coord: Coord, state: Id, neighbor_state: Id) {}
    fn light_updated(&self, world: &mut VoxelWorld, coord: Coord, old_level: u8, new_level: u8) {}
    fn call(&self, world: &mut VoxelWorld, coord: Coord, state: Id, function: &str, arg: Tag) -> Tag { Tag::Null }
    fn on_entity_collide(&self, world: &mut VoxelWorld, coord: Coord, state: Id, face: Direction, entity: ()) {}
    fn on_interact(&self, world: &mut VoxelWorld, coord: Coord, state: Id) {}
    fn on_update(&self, world: &mut VoxelWorld, coord: Coord, state: Id) {}
    fn on_place(&self, world: &mut VoxelWorld, context: &mut PlaceContext) { }
    fn on_remove(&self, world: &mut VoxelWorld, coord: Coord, old: Id, new: Id) {}
    fn on_data_set(&self, world: &mut VoxelWorld, coord: Coord, state: Id, data: &mut Tag) {}
    fn on_data_delete(&self, world: &mut VoxelWorld, coord: Coord, state: Id, data: Tag) {}
    fn on_enabled_changed(&self, world: &mut VoxelWorld, coord: Coord, state: Id, enabled: bool) {}
    fn push_mesh(&self, mesh_builder: &mut MeshBuilder, level_of_detail: LOD, world: &VoxelWorld, coord: Coord, state: Id, occlusion: Occlusion, orientation: Orientation) {}
    // fn rotate(&self, coord: Coord, state: Id, rotation: Rotation) -> Id { state }
    fn default_state(&self) -> BlockState;
    fn raycast(&self, ray: Ray3d, world: &VoxelWorld, coord: Coord, state: Id, orientation: Orientation) -> Option<f32> {
        let aabb = AABB::voxel(coord);
        aabb.intersects(ray)
    }
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