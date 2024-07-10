pub mod meshing;
pub mod block;
pub mod occlusion_shape;
pub mod faces;
pub mod lighting;
pub mod coord;
pub mod direction;
pub mod world;
pub mod engine;
pub mod blockregistry;
pub mod blockstate;
pub mod blocks;

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::{blockstate, core::voxel::block::Block};

    use super::blocks;

    #[test]
    fn registry_test() {
        struct AirBlock;
        struct DirtBlock;
        struct StoneBlock;
        impl Block for AirBlock {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn name(&self) -> &str {
                "air"
            }

            fn default_state(&self) -> super::blockstate::BlockState {
                blockstate!(air)
            }
        }
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
        let air_block = blocks::register_block(AirBlock);
        let dirt_block = blocks::register_block(DirtBlock);
        let stone_block = blocks::register_block(StoneBlock);
        let air = blocks::register_state(air_block.default_state());
        let dirt = blocks::register_state(dirt_block.default_state());
        let stone = blocks::register_state(stone_block.default_state());
        let test = blocks::register_state(blockstate!(air, test = "Hello, world"));
        let test2 = blocks::register_state(blockstate!(air, test = "test"));
        let test3 = blocks::register_state(blockstate!(air, test = "test3"));
        
        println!("{}", air);
        println!("{}", test2);
    }
}