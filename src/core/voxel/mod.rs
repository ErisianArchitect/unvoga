#![allow(unused)]
// pub mod meshing;
pub mod block;
pub mod occlusionshape;
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
pub mod faceflags;
pub mod statevalue;
pub mod blocklayer;
pub mod procgen;

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::{blockstate, core::voxel::{block::Block, blocks::Id, coord::Coord, world::VoxelWorld}};

    use super::blocks;

    #[test]
    fn registry_test() {
        struct DirtBlock;
        struct StoneBlock;
        impl Block for DirtBlock {

            fn name(&self) -> &str {
                "dirt"
            }

            fn default_state(&self) -> super::blockstate::BlockState {
                blockstate!(dirt)
            }
        }

        impl Block for StoneBlock {

            fn name(&self) -> &str {
                "stone"
            }
            fn default_state(&self) -> super::blockstate::BlockState {
                blockstate!(stone)
            }
        }
        let dirt_block = blocks::register_block(DirtBlock);
        let stone_block = blocks::register_block(StoneBlock);
        let air = Id::AIR;
        let dirt = blocks::register_state(dirt_block.default_state());
        let stone = blocks::register_state(stone_block.default_state());
        let test = blocks::register_state(blockstate!(air, test = "Hello, world"));
        let test2 = blocks::register_state(blockstate!(air, test = "test"));
        let test3 = blocks::register_state(blockstate!(air, test = "test3"));
        // let mut world = World {};
        // world.set_block(Coord::new(1, 2, 3), air);
        println!("{}", air.block().name());
        // println!("{}", air.block().light_args(&world, (0, 0, 0).into(), air).filter());
        println!("{}", test2);
    }
}