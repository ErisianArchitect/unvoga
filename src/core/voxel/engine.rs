#![allow(unused)]
use std::{borrow::Borrow, cell::{Ref, RefCell, RefMut}};

use crate::{blockstate, core::voxel::{block::Block, coord::Coord, direction::{Cardinal, Direction}, faces::Faces, lighting::lightargs::LightArgs, occlusionshape::OcclusionShape}};
//
use super::world::VoxelWorld;

pub struct VoxelEngine {
    // blocks: RefCell<BlockRegistry>,
    world: RefCell<VoxelWorld>,
    event_queue: (),
}

impl VoxelEngine {
    // pub fn blocks(&self) -> Ref<BlockRegistry> {
    //     self.blocks.borrow()
    // }

    // pub fn blocks_mut(&self) -> RefMut<BlockRegistry> {
    //     self.blocks.borrow_mut()
    // }

    pub fn world(&self) -> Ref<VoxelWorld> {
        self.world.borrow()
    }

    pub fn world_mut(&self) -> RefMut<VoxelWorld> {
        self.world.borrow_mut()
    }
}

// #[test]
// pub fn engine_test() {
//     let mut engine = VoxelEngine {
//         blocks: RefCell::new(BlockRegistry::default()),
//         world: RefCell::new(World {}),
//         event_queue: ()
//     };
//     struct AirBlock;
//     struct DirtBlock;
//     struct StoneBlock;
//     struct DoorBlock {
//         top_id: Option<StateId>,
//     }
//     impl Block for AirBlock {
//         fn as_any(&self) -> &dyn std::any::Any {
//             self
//         }

//         fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//             self
//         }

//         fn name(&self) -> &str {
//             "air"
//         }

//         fn light_args(&self) -> super::lighting::lightargs::LightArgs {
//             LightArgs::new(1, 0)
//         }
        
//         fn occlusion_shapes(&self) -> &super::faces::Faces<super::occlusion_shape::OcclusionShape> {
//             const FACE_SHAPES: Faces<OcclusionShape> = Faces {
//                 neg_x: OcclusionShape::None,
//                 neg_y: OcclusionShape::None,
//                 neg_z: OcclusionShape::None,
//                 pos_x: OcclusionShape::None,
//                 pos_y: OcclusionShape::None,
//                 pos_z: OcclusionShape::None,
//             };
//             &FACE_SHAPES
//         }
//     }
//     impl Block for DirtBlock {
//         fn as_any(&self) -> &dyn std::any::Any {
//             self
//         }

//         fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//             self
//         }
//         fn name(&self) -> &str {
//             "dirt"
//         }
//     }
//     impl Block for StoneBlock {
//         fn as_any(&self) -> &dyn std::any::Any {
//             self
//         }

//         fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//             self
//         }
//         fn name(&self) -> &str {
//             "stone"
//         }
//     }
//     impl Block for DoorBlock {
//         fn as_any(&self) -> &dyn std::any::Any {
//             self
//         }

//         fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//             self
//         }
//         fn name(&self) -> &str {
//             "door"
//         }

//         fn on_place(
//                 &self,
//                 registry: &BlockRegistry,
//                 world: &mut World,
//                 coord: Coord,
//                 state: &super::blockstate::BlockState,
//             ) {
//             if state["bottom"] == State::Bool(true) {
//                 // let mut world = engine.world.borrow_mut();
//                 let mut top_state = state.clone();
//                 top_state["bottom"] = State::Bool(false);
//                 // let top_id = engine.blocks_mut().register_state(top_state);
//                 // let blocks = engine.blocks();
//                 let block = registry.get_state(self.top_id.expect("top_id not set"));
//                 // println!("block: {}", block.name());
//                 println!("Bottom placed at {coord}");
//                 world.set_block(registry, coord + Direction::PosY, self.top_id.expect("top_id not set"));
//             } else {
//                 println!("Top placed at {coord}");
//                 // let id = registry.register_state(blockstate!(stone, test="Hello, world"));
//                 // world.set_block(registry, coord + Direction::PosY, id);
//             }
//         }
//     }
//     engine.blocks_mut().register_block(AirBlock);
//     engine.blocks_mut().register_block(DirtBlock);
//     engine.blocks_mut().register_block(StoneBlock);
//     let door_block = engine.blocks_mut().register_block(DoorBlock { top_id: None } );
//     let air = engine.blocks_mut().register_state(blockstate!(air));
//     let dirt = engine.blocks_mut().register_state(blockstate!(dirt));
//     let stone = engine.blocks_mut().register_state(blockstate!(stone));
//     let bottom_door = engine.blocks_mut().register_state(blockstate!(door, bottom = true, open = false, direction = Cardinal::East, open_left = true));
//     let top_door = engine.blocks_mut().register_state(blockstate!(door, bottom = false, open = false, direction = Cardinal::East, open_left = true));
//     {
//         let mut blocks = engine.blocks_mut();
//         // let name = blocks.get_block(door_block).name();
//         // println!("Block name: {name}");
//         let door: &mut DoorBlock = blocks.get_block_cast_mut(door_block);
//         door.top_id = Some(top_door);
//     }

//     let blocks = engine.blocks();
//     let (state, block) = blocks.get_state_and_block(bottom_door);
//     engine.world_mut().set_block(&*engine.blocks.borrow(), Coord::new(0, 0, 0), bottom_door);
//     let door: &DoorBlock = blocks.get_block_cast(door_block);
//     println!("Door");
//     // block.on_place(&engine, Coord::new(1, 2, 3), state);
// }