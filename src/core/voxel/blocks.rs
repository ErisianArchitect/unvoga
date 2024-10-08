#![allow(unused)]
use std::{borrow::Borrow, ops::{Deref, Index}, sync::{atomic::{AtomicBool, Ordering}, OnceLock}};

use bevy::{math::Ray3d, utils::hashbrown::HashMap};

use crate::{blockstate, core::voxel::blockstate};

use super::{block::Block, statevalue::StateValue, blockstate::BlockState, coord::Coord, lighting::lightargs::LightArgs, occluder::Occluder, occlusionshape::OcclusionShape, world::VoxelWorld};

struct RegistryEntry {
    state: BlockState,
    block_ref: BlockId,
}

static mut STATES: OnceLock<Vec<RegistryEntry>> = OnceLock::new();
static mut BLOCKS: OnceLock<Vec<Box<dyn Block>>> = OnceLock::new();
static mut ID_LOOKUP: OnceLock<HashMap<BlockState, Id>> = OnceLock::new();
static mut BLOCK_LOOKUP: OnceLock<HashMap<String, BlockId>> = OnceLock::new();
static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Returns true if initialization occurred.
/// You don't really need to call this function since every other function calls it.
fn init() -> bool {
    unsafe {
        if INITIALIZED.swap(true, Ordering::SeqCst) {
            return false;
        }
        STATES.set(Vec::with_capacity(4096));
        BLOCKS.set(Vec::with_capacity(512));
        ID_LOOKUP.set(HashMap::new());
        BLOCK_LOOKUP.set(HashMap::new());
        register_block(AirBlock);
        register_state(blockstate!(air));
        true
    }
}

#[must_use]
pub fn register_state<B: Borrow<BlockState>>(state: B) -> Id {
    init();
    unsafe {
        let id_lookup = ID_LOOKUP.get_mut().expect("Failed to get");
        if let Some(&id) = id_lookup.get(state.borrow()) {
            return id;
        }
        let state: BlockState = state.borrow().clone();
        let block_lookup = BLOCK_LOOKUP.get().expect("Failed to get");
        let block_id = if let Some(&block_id) = block_lookup.get(state.name()) {
            block_id
        } else {
            panic!("Block not found: {}", state.name());
        };
        let states = STATES.get_mut().expect("Failed to get");
        let id = states.len() as u32;
        id_lookup.insert(state.clone(), Id(id));
        states.push(RegistryEntry { block_ref: block_id, state });
        Id(id)
    }
}

#[must_use]
pub fn register_block<B: Block>(mut block: B) -> BlockId {
    init();
    unsafe {
        let block_lookup = BLOCK_LOOKUP.get_mut().expect("Failed to get");
        if block_lookup.contains_key(block.name()) {
            panic!("Block already registered: {}", block.name());
        }
        let blocks = BLOCKS.get_mut().expect("Failed to get");
        let id = blocks.len() as u32;
        block_lookup.insert(block.name().to_owned(), BlockId(id));
        block.on_register();
        blocks.push(Box::new(block));
        BlockId(id)
    }
}

/// If the [BlockState] has already been registered, find the associated [Id].

#[must_use]
pub fn find_state<B: Borrow<BlockState>>(state: B) -> Option<Id> {
    init();
    unsafe {
        let id_lookup = ID_LOOKUP.get().expect("Failed to get");
        id_lookup.get(state.borrow()).map(|&id| id)
    }
}


#[must_use]
pub fn find_block<S: AsRef<str>>(name: S) -> Option<BlockId> {
    init();
    unsafe {
        let block_lookup = BLOCK_LOOKUP.get().expect("Failed to get block lookup");
        block_lookup.get(name.as_ref()).map(|&id| id)
    }
}


#[must_use]
pub fn get_block_ref(id: Id) -> BlockId {
    unsafe {
        let states = STATES.get().expect("Failed to get states");
        states[id.0 as usize].block_ref
    }
}


#[must_use]
pub fn get_state(id: Id) -> &'static BlockState {
    // Id is only issued by the registry, so this doesn't need
    // to call init because it can be assumed that init has already
    // been called.
    // It can also be assumed that Id is associated with a BlockState
    unsafe {
        let states = STATES.get().expect("Failed to get states");
        &states[id.0 as usize].state
    }
}


#[must_use]
pub fn get_block(id: BlockId) -> &'static dyn Block {
    // BlockRef is only issued by the registry, so this doesn't need
    // to call init because it can be assumed that init has already
    // been called.
    // It can also be assumed that BlockRef is associated with a Block.
    unsafe {
        let blocks = BLOCKS.get().expect("Failed to get blocks");
        blocks[id.0 as usize].as_ref()
    }
}


#[must_use]
pub fn get_block_for(id: Id) -> &'static dyn Block {
    unsafe {
        let states = STATES.get().expect("Failed to get states");
        let block_id = states[id.0 as usize].block_ref;
        let blocks = BLOCKS.get().expect("Failed to get blocks");
        blocks[block_id.0 as usize].as_ref()
    }
}


#[must_use]
pub fn get_state_and_block(id: Id) -> (&'static BlockState, &'static dyn Block) {
    unsafe {
        let states = STATES.get().expect("Failed to get states");
        let block_id = states[id.0 as usize].block_ref;
        let state = &states[id.0 as usize].state;
        let blocks = BLOCKS.get().expect("Failed to get blocks");
        let block = blocks[block_id.0 as usize].as_ref();
        (state, block)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(u32);

impl Id {
    pub const AIR: Self = Id(0);
    /// Make sure you don't register any states while this reference is held.
    
    #[must_use]
    pub unsafe fn unsafe_state(self) -> &'static BlockState {
        get_state(self)
    }

    /// Make sure you don't register any blocks while this reference is held.
    
    #[must_use]
    pub unsafe fn unsafe_block(self) -> &'static dyn Block {
        get_block_for(self)
    }

    
    #[must_use]
    pub fn block(self) -> BlockId {
        get_block_ref(self)
    }

    /// Returns true if this block is not air.
    
    pub fn is_non_air(self) -> bool {
        self.0 != 0
    }

    /// Don't register anything while these references are held.
    
    #[must_use]
    pub unsafe fn unsafe_state_and_block(self) -> (&'static BlockState, &'static dyn Block) {
        get_state_and_block(self)
    }

    
    pub fn id(self) -> u32 {
        self.0
    }

    
    pub fn block_id(self) -> u32 {
        get_block_ref(self).id()
    }

    
    pub fn is_air(self) -> bool {
        self.0 == 0
    }

    
    #[must_use]
    pub fn clone_state(self) -> BlockState {
        (*self).clone()
    }
}

// impl Borrow<BlockState> for Id {
//     fn borrow(&self) -> &BlockState {
//         &**self
//     }
// }

impl AsRef<BlockState> for Id {
    fn as_ref(&self) -> &BlockState {
        &**self
    }
}

impl<S: AsRef<str>> Index<S> for Id {
    type Output = StateValue;
    
    fn index(&self, index: S) -> &Self::Output {
        const NULL: StateValue = StateValue::Null;
        self.get_property(index).unwrap_or(&NULL)
    }
}

impl BlockId {
    
    pub unsafe fn unsafe_block(self) -> &'static dyn Block {
        get_block(self)
    }

    
    pub fn id(self) -> u32 {
        self.0
    }
}

impl Deref for Id {
    type Target = BlockState;

    
    fn deref(&self) -> &Self::Target {
        unsafe {
            let states = STATES.get().expect("Failed to get");
            &states[self.0 as usize].state
        }
    }
}

impl Deref for BlockId {
    type Target = dyn Block;

    
    fn deref(&self) -> &Self::Target {
        unsafe {
            let blocks = BLOCKS.get().expect("Failed to get");
            blocks[self.0 as usize].as_ref()
        }
    }
}

impl std::fmt::Display for Id {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_fmt(f)
    }
}

pub struct AirBlock;

impl Block for AirBlock {

    fn raycast(&self, ray: Ray3d, world: &VoxelWorld, coord: Coord, state: Id, orientation: crate::prelude::Orientation) -> Option<f32> {
        None
    }
    
    fn name(&self) -> &str {
        "air"
    }
    
    fn light_args(&self, world: &VoxelWorld, coord: Coord, state: Id) -> LightArgs {
        LightArgs::new(1, 0)
    }

    
    fn occluder(&self, world: &VoxelWorld, state: Id) -> &Occluder {
        &Occluder::EMPTY_FACES
    }

    fn default_state(&self) -> BlockState {
        blockstate!(air)
    }
}