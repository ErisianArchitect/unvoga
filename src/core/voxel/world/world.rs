
use crate::core::voxel::{blockregistry::{BlockRegistry, StateId}, blocks::StateRef, coord::Coord, engine::VoxelEngine};
pub struct World {
    
}

impl World {
    pub fn set_block(&mut self, coord: Coord, id: StateRef) -> StateRef {
        // let blocks = engine.blocks();
        unsafe {
            // let (state, block) = id.state_and_block();
            // println!("Block Name: {}", block.name());
            println!("Set Block: {coord} = {id} : {}", id.block().name());
            id.block().on_place(self, coord, id);
        }
        id
    }
}