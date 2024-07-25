#![allow(unused)]
use std::{borrow::Borrow, ops::{Deref, Index}, sync::{atomic::{AtomicBool, Ordering}, Arc, LazyLock, Mutex, OnceLock}};

use hashbrown::HashMap;

use crate::{blockstate, core::voxel::blockstate};

use super::{block::Block, blockstate::{BlockState, StateValue}, coord::Coord, lighting::lightargs::LightArgs, occluder::Occluder, occlusion_shape::OcclusionShape, world::VoxelWorld};

struct RegistryEntry {
    state: Arc<BlockState>,
    block_ref: BlockId,
}

impl RegistryEntry {
    pub fn new(state: BlockState, block_ref: BlockId) -> Self {
        Self {
            state: Arc::new(state),
            block_ref
        }
    }
}

static STATES: LazyLock<Mutex<Vec<RegistryEntry>>> = LazyLock::new(|| Mutex::new(vec![
    RegistryEntry::new(
        blockstate!(air),
        BlockId(0)
    )
]));
static BLOCKS: LazyLock<Mutex<Vec<Arc<dyn Block>>>> = LazyLock::new(|| Mutex::new(vec![
    Arc::new(AirBlock)
]));
static ID_LOOKUP: LazyLock<Mutex<HashMap<BlockState, Id>>> = LazyLock::new(|| Mutex::new(HashMap::from([
    (blockstate!(air), Id(0))
])));
static BLOCK_LOOKUP: LazyLock<Mutex<HashMap<String, BlockId>>> = LazyLock::new(|| Mutex::new(HashMap::from([
    (String::from("air"), BlockId(0))
])));

// static mut STATES: OnceLock<Vec<RegistryEntry>> = OnceLock::new();
// static mut BLOCKS: OnceLock<Vec<Box<dyn Block>>> = OnceLock::new();
// static mut ID_LOOKUP: OnceLock<HashMap<BlockState, Id>> = OnceLock::new();
// static mut BLOCK_LOOKUP: OnceLock<HashMap<String, BlockId>> = OnceLock::new();
// static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Returns true if initialization occurred.
/// You don't really need to call this function since every other function calls it.
// fn init() -> bool {
//     unsafe {
//         if INITIALIZED.swap(true, Ordering::SeqCst) {
//             return false;
//         }
//         STATES.set(Vec::with_capacity(4096));
//         BLOCKS.set(Vec::with_capacity(512));
//         ID_LOOKUP.set(HashMap::new());
//         BLOCK_LOOKUP.set(HashMap::new());
//         register_block(AirBlock);
//         register_state(blockstate!(air));
//         true
//     }
// }

#[must_use]
pub fn register_state<B: Borrow<BlockState>>(state: B) -> Id {
    let mut id_lookup = ID_LOOKUP.lock().expect("Failed to lock ID_LOOKUP");
    if let Some(&id) = id_lookup.get(state.borrow()) {
        return id;
    }
    let state: BlockState = state.borrow().clone();
    let block_lookup = BLOCK_LOOKUP.lock().expect("Failed to lock BLOCK_LOOKUP");
    let block_id = if let Some(&block_id) = block_lookup.get(state.name()) {
        block_id
    } else {
        panic!("Block not found: {}", state.name());
    };
    let mut states = STATES.lock().expect("Failed to lock STATES");
    let id = states.len() as u32;
    id_lookup.insert(state.clone(), Id(id));
    states.push(RegistryEntry::new(state, block_id));
    Id(id)
}

#[must_use]
pub fn register_block<B: Block>(block: B) -> BlockId {
    let mut block_lookup = BLOCK_LOOKUP.lock().expect("Failed to lock BLOCK_LOOKUP");
    if block_lookup.contains_key(block.name()) {
        panic!("Block already registered: {}", block.name());
    }
    let mut blocks = BLOCKS.lock().expect("Failed to lock BLOCKS");
    let id = blocks.len() as u32;
    block_lookup.insert(block.name().to_owned(), BlockId(id));
    blocks.push(Arc::new(block));
    BlockId(id)
}

/// If the [BlockState] has already been registered, find the associated [Id].

#[must_use]
pub fn find_state<B: Borrow<BlockState>>(state: B) -> Option<Id> {
    let id_lookup = ID_LOOKUP.lock().expect("Failed to lock ID_LOOKUP");
    id_lookup.get(state.borrow()).map(|&id| id)
}


#[must_use]
pub fn find_block<S: AsRef<str>>(name: S) -> Option<BlockId> {
    let block_lookup = BLOCK_LOOKUP.lock().expect("Failed to lock BLOCK_LOOKUP");
    block_lookup.get(name.as_ref()).map(|&id| id)
}


#[must_use]
pub fn get_block_ref(id: Id) -> BlockId {
    let states = STATES.lock().expect("Failed to lock STATES");
    states[id.0 as usize].block_ref
}


#[must_use]
pub fn get_state(id: Id) -> Arc<BlockState> {
    // Id is only issued by the registry, so this doesn't need
    // to call init because it can be assumed that init has already
    // been called.
    // It can also be assumed that Id is associated with a BlockState
    let states = STATES.lock().expect("Failed to lock STATES");
    states[id.0 as usize].state.clone()
}


#[must_use]
pub fn get_block(id: BlockId) -> Arc<dyn Block> {
    // BlockRef is only issued by the registry, so this doesn't need
    // to call init because it can be assumed that init has already
    // been called.
    // It can also be assumed that BlockRef is associated with a Block.
    let blocks = BLOCKS.lock().expect("Failed to lock BLOCKS");
    blocks[id.0 as usize].clone()
}


#[must_use]
pub fn get_block_for(id: Id) -> Arc<dyn Block> {
    let states = STATES.lock().expect("Failed to lock STATES");
    let block_id = states[id.0 as usize].block_ref;
    let blocks = BLOCKS.lock().expect("Failed to lock BLOCKS");
    blocks[block_id.0 as usize].clone()
}


#[must_use]
pub fn get_state_and_block(id: Id) -> (Arc<BlockState>, Arc<dyn Block>) {
    let states = STATES.lock().expect("Failed to lock STATES");
    let block_id = states[id.0 as usize].block_ref;
    let state = states[id.0 as usize].state.clone();
    let blocks = BLOCKS.lock().expect("Failed to lock BLOCKS");
    let block = blocks[block_id.0 as usize].clone();
    (state, block)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(u32);

impl Id {
    pub const AIR: Self = Id(0);
    /// Make sure you don't register any states while this reference is held.
    
    // #[must_use]
    // pub unsafe fn unsafe_state(self) -> &'static BlockState {
    //     get_state(self)
    // }

    /// Make sure you don't register any blocks while this reference is held.
    
    // #[must_use]
    // pub unsafe fn unsafe_block(self) -> &'static dyn Block {
    //     get_block_for(self)
    // }

    
    #[must_use]
    pub fn block(self) -> BlockId {
        get_block_ref(self)
    }

    /// Returns true if this block is not air.
    
    pub fn is_non_air(self) -> bool {
        self.0 != 0
    }

    /// Don't register anything while these references are held.
    
    // #[must_use]
    // pub unsafe fn unsafe_state_and_block(self) -> (&'static BlockState, &'static dyn Block) {
    //     get_state_and_block(self)
    // }

    
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
    
    // pub unsafe fn unsafe_block(self) -> &'static dyn Block {
    //     get_block(self)
    // }

    
    pub fn id(self) -> u32 {
        self.0
    }
}

impl Deref for Id {
    type Target = BlockState;

    
    fn deref(&self) -> &Self::Target {
        unsafe {
            let states = STATES.lock().expect("Failed to get");
            states[self.0 as usize].state.deref()
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

#[derive(Debug)]
pub struct AirBlock;

impl Block for AirBlock {
    
    fn name(&self) -> &str {
        "air"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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