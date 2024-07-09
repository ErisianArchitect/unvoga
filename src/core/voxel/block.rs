use super::{coord::Coord, direction::Direction, faces::Faces, lighting::lightargs::LightArgs, occlusion_shape::OcclusionShape};

pub trait Block {
    fn occlusion_shapes(&self) -> &Faces<OcclusionShape>;
    fn light_args() -> LightArgs;
    fn neighbor_updated(
        &self,
        world: &mut (/* Gotta make the world first */),
        coord: Coord,
        neighbor_coord: Coord,
        direction: Direction,
    );
    fn on_place(
        &self,
        world: &mut (),
        coord: Coord,
        state: (),
    );
    fn on_remove(
        &self,
        world: &mut (),
        coord: Coord,
        state: (),
    );
    fn push_mesh(
        &self,
        mesh_builder: &mut (),
        coord: Coord,
        state: (),
    );

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