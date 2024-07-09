use std::cell::RefCell;

use super::{blockregistry::BlockRegistry, world::world::World};

pub struct VoxelEngine {
    blocks: BlockRegistry,
    world: RefCell<World>,
    event_queue: (),
}