use std::any::Any;
use super::{blockregistry::BlockRegistry, blocks::StateRef, blockstate::BlockState, coord::Coord, direction::Direction, engine::VoxelEngine, faces::Faces, lighting::lightargs::LightArgs, occlusion_shape::OcclusionShape, world::world::World};

pub trait Block: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn name(&self) -> &str;
    fn occlusion_shapes(&self) -> &Faces<OcclusionShape> {
        const FULL_FACES: Faces<OcclusionShape> = Faces {
            neg_x: OcclusionShape::Full,
            neg_y: OcclusionShape::Full,
            neg_z: OcclusionShape::Full,
            pos_x: OcclusionShape::Full,
            pos_y: OcclusionShape::Full,
            pos_z: OcclusionShape::Full,
        };
        &FULL_FACES
    }
    fn light_args(&self) -> LightArgs { LightArgs::new(15, 0) }
    fn neighbor_updated(
        &self,
        world: &mut World,
        coord: Coord,
        neighbor_coord: Coord,
        direction: Direction,
    ) {}
    fn on_place(
        &self,
        world: &mut World,
        coord: Coord,
        state: StateRef,
    ) {}
    fn on_remove(
        &self,
        world: &mut World,
        coord: Coord,
        state: StateRef,
    ) {}
    fn push_mesh(
        &self,
        mesh_builder: &mut (),
        coord: Coord,
        state: StateRef,
    ) {}
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