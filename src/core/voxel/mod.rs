pub mod meshing;
pub mod block;
pub mod occlusion_shape;
pub mod faces;
pub mod lighting;
pub mod coord;
pub mod direction;
pub mod world;
pub mod engine;
// pub mod blockregistry;
pub mod blockstate;
pub mod blocks;
pub mod rendering;
pub mod tag;
pub mod axis;
pub mod occluder;
pub mod region;

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::{blockstate, core::voxel::{block::Block, blocks::StateRef, coord::Coord, world::VoxelWorld}};

    use super::blocks;

    #[test]
    fn registry_test() {
        struct DirtBlock;
        struct StoneBlock;
        impl Block for DirtBlock {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn name(&self) -> &str {
                "dirt"
            }

            fn default_state(&self) -> super::blockstate::BlockState {
                blockstate!(dirt)
            }
        }

        impl Block for StoneBlock {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn name(&self) -> &str {
                "stone"
            }
            fn default_state(&self) -> super::blockstate::BlockState {
                blockstate!(stone)
            }
        }
        let dirt_block = blocks::register_block(DirtBlock);
        let stone_block = blocks::register_block(StoneBlock);
        let air = StateRef::AIR;
        let dirt = blocks::register_state(dirt_block.default_state());
        let stone = blocks::register_state(stone_block.default_state());
        let test = blocks::register_state(blockstate!(air, test = "Hello, world"));
        let test2 = blocks::register_state(blockstate!(air, test = "test"));
        let test3 = blocks::register_state(blockstate!(air, test = "test3"));
        // let mut world = World {};
        // world.set_block(Coord::new(1, 2, 3), air);
        println!("{}", air.block().name());
        println!("{}", air.block().light_args(air).filter());
        println!("{}", test2);
    }
}