use std::{borrow::Borrow, ops::{Deref, Index}, sync::{atomic::{AtomicBool, Ordering}, OnceLock}};

use bevy::utils::hashbrown::HashMap;

use crate::{blockstate, core::voxel::blockstate};

use super::{block::Block, blockstate::{BlockState, State}, lighting::lightargs::LightArgs, occlusion_shape::OcclusionShape};

struct RegistryEntry {
    state: BlockState,
    block_ref: BlockRef,
}

static mut STATES: OnceLock<Vec<RegistryEntry>> = OnceLock::new();
static mut BLOCKS: OnceLock<Vec<Box<dyn Block>>> = OnceLock::new();
static mut ID_LOOKUP: OnceLock<HashMap<BlockState, StateRef>> = OnceLock::new();
static mut BLOCK_LOOKUP: OnceLock<HashMap<String, BlockRef>> = OnceLock::new();
static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Returns true if initialization occurred.
/// You don't really need to call this function since every other function calls it.
fn init() -> bool {
    unsafe {
        if INITIALIZED.swap(true, Ordering::SeqCst) {
            return false;
        }
        STATES.set(Vec::new());
        BLOCKS.set(Vec::new());
        ID_LOOKUP.set(HashMap::new());
        BLOCK_LOOKUP.set(HashMap::new());
        register_block(AirBlock);
        register_state(blockstate!(air));
        true
    }
}

pub fn register_state<B: Borrow<BlockState>>(state: B) -> StateRef {
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
        id_lookup.insert(state.clone(), StateRef(id));
        states.push(RegistryEntry { block_ref: block_id, state });
        StateRef(id)
    }
}

pub fn register_block<B: Block>(block: B) -> BlockRef {
    init();
    unsafe {
        let block_lookup = BLOCK_LOOKUP.get_mut().expect("Failed to get");
        if block_lookup.contains_key(block.name()) {
            panic!("Block already registered: {}", block.name());
        }
        let blocks = BLOCKS.get_mut().expect("Failed to get");
        let id = blocks.len() as u32;
        block_lookup.insert(block.name().to_owned(), BlockRef(id));
        blocks.push(Box::new(block));
        BlockRef(id)
    }
}

/// If the [BlockState] has already been registered, find the associated [StateRef].
pub fn find_state<B: Borrow<BlockState>>(state: B) -> Option<StateRef> {
    if init() {
        return None;
    }
    unsafe {
        let id_lookup = ID_LOOKUP.get().expect("Failed to get");
        id_lookup.get(state.borrow()).map(|&id| id)
    }
}

pub fn find_block<S: AsRef<str>>(name: S) -> Option<BlockRef> {
    if init() {
        return None;
    }
    unsafe {
        let block_lookup = BLOCK_LOOKUP.get().expect("Failed to get");
        block_lookup.get(name.as_ref()).map(|&id| id)
    }
}

pub fn get_block_ref(id: StateRef) -> BlockRef {
    unsafe {
        let states = STATES.get().expect("Failed to get");
        states[id.0 as usize].block_ref
    }
}

pub fn get_state(id: StateRef) -> &'static BlockState {
    // StateRef is only issued by the registry, so this doesn't need
    // to call init because it can be assumed that init has already
    // been called.
    // It can also be assumed that StateRef is associated with a BlockState
    unsafe {
        let states = STATES.get().expect("Failed to get");
        &states[id.0 as usize].state
    }
}

pub fn get_block(id: BlockRef) -> &'static dyn Block {
    // BlockRef is only issued by the registry, so this doesn't need
    // to call init because it can be assumed that init has already
    // been called.
    // It can also be assumed that BlockRef is associated with a Block.
    unsafe {
        let blocks = BLOCKS.get().expect("Failed to get");
        blocks[id.0 as usize].as_ref()
    }
}

pub fn get_block_for(id: StateRef) -> &'static dyn Block {
    unsafe {
        let states = STATES.get().expect("Failed to get");
        let block_id = states[id.0 as usize].block_ref;
        let blocks = BLOCKS.get().expect("Failed to get");
        blocks[block_id.0 as usize].as_ref()
    }
}

pub fn get_state_and_block(id: StateRef) -> (&'static BlockState, &'static dyn Block) {
    unsafe {
        let states = STATES.get().expect("Failed to get");
        let block_id = states[id.0 as usize].block_ref;
        let state = &states[id.0 as usize].state;
        let blocks = BLOCKS.get().expect("Failed to get");
        let block = blocks[block_id.0 as usize].as_ref();
        (state, block)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateRef(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockRef(u32);

impl StateRef {
    pub const AIR: Self = StateRef(0);
    /// Make sure you don't register any states while this reference is held.
    pub unsafe fn unsafe_state(self) -> &'static BlockState {
        get_state(self)
    }

    /// Make sure you don't register any blocks while this reference is held.
    pub unsafe fn unsafe_block(self) -> &'static dyn Block {
        get_block_for(self)
    }

    pub fn block(self) -> BlockRef {
        get_block_ref(self)
    }

    /// Don't register anything while these references are held.
    pub unsafe fn unsafe_state_and_block(self) -> (&'static BlockState, &'static dyn Block) {
        get_state_and_block(self)
    }

    pub fn id(self) -> u32 {
        self.0
    }

    pub fn is_air(self) -> bool {
        self.0 == 0
    }
}

impl<S: AsRef<str>> Index<S> for StateRef {
    type Output = State;

    fn index(&self, index: S) -> &Self::Output {
        const NULL: State = State::Null;
        self.get_property(index).unwrap_or(&NULL)
    }
}

impl BlockRef {
    pub unsafe fn unsafe_block(self) -> &'static dyn Block {
        get_block(self)
    }

    pub fn id(self) -> u32 {
        self.0
    }
}

impl Deref for StateRef {
    type Target = BlockState;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let states = STATES.get().expect("Failed to get");
            &states[self.0 as usize].state
        }
    }
}

impl Deref for BlockRef {
    type Target = dyn Block;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let blocks = BLOCKS.get().expect("Failed to get");
            blocks[self.0 as usize].as_ref()
        }
    }
}

impl std::fmt::Display for StateRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_fmt(f)
    }
}

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

    fn light_args(&self) -> LightArgs {
        LightArgs::new(1, 0)
    }

    fn occlusion_shapes(&self) -> &super::faces::Faces<super::occlusion_shape::OcclusionShape> {
        &OcclusionShape::EMPTY_FACES
    }

    fn default_state(&self) -> BlockState {
        blockstate!(air)
    }
}