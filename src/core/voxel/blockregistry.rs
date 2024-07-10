use std::cell::Ref;
use std::{any::*, cell::RefCell};
use std::borrow::Borrow;

use bevy::utils::hashbrown::HashMap;

use crate::{blockstate, core::voxel::{coord::Coord, direction::Cardinal, engine::VoxelEngine, faces::Faces, lighting::lightargs::LightArgs, occlusion_shape::OcclusionShape, world::world::World}};

use super::{block::Block, blockstate::BlockState};

pub mod registry {
    use crate::core::voxel::blockstate::State;

    use super::*;
    use std::{ops::{Deref, Index}, sync::{atomic::{AtomicBool, Ordering}, OnceLock}};

    struct RegistryEntry {
        state: BlockState,
        block_id: BlockRef,
    }

    static mut STATES: OnceLock<Vec<RegistryEntry>> = OnceLock::new();
    static mut BLOCKS: OnceLock<Vec<Box<dyn Block>>> = OnceLock::new();
    static mut ID_LOOKUP: OnceLock<HashMap<BlockState, StateRef>> = OnceLock::new();
    static mut BLOCK_LOOKUP: OnceLock<HashMap<String, BlockRef>> = OnceLock::new();
    static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

    /// Returns true if initialization occurred.
    /// You don't really need to call this function since every other function calls it.
    pub fn init() -> bool {
        unsafe {
            if INITIALIZED.swap(true, Ordering::SeqCst) {
                return false;
            }
            STATES.set(Vec::new());
            BLOCKS.set(Vec::new());
            ID_LOOKUP.set(HashMap::new());
            BLOCK_LOOKUP.set(HashMap::new());
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
            states.push(RegistryEntry { block_id, state });
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
            let block_id = states[id.0 as usize].block_id;
            let blocks = BLOCKS.get().expect("Failed to get");
            blocks[block_id.0 as usize].as_ref()
        }
    }

    pub fn get_state_and_block(id: StateRef) -> (&'static BlockState, &'static dyn Block) {
        unsafe {
            let states = STATES.get().expect("Failed to get");
            let block_id = states[id.0 as usize].block_id;
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
        pub fn state(self) -> &'static BlockState {
            get_state(self)
        }

        pub fn block(self) -> &'static dyn Block {
            get_block_for(self)
        }

        pub fn state_and_block(self) -> (&'static BlockState, &'static dyn Block) {
            get_state_and_block(self)
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
        pub fn get(self) -> &'static dyn Block {
            get_block(self)
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
}

struct RegistryEntry {
    state: BlockState,
    block_id: u32,
}

struct BlockEntry {
    name: String,
    block: Box<dyn Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(u32);

impl StateId {
    pub fn id(self) -> u32 {
        self.0
    }
}

impl BlockId {
    pub fn id(self) -> u32 {
        self.0
    }
}

/// All states must have an associated block.
/// Register blocks before registering states.
pub struct BlockRegistry {
    states: RefCell<Vec<RegistryEntry>>,
    blocks: Vec<Box<dyn Block>>,
    id_lookup: RefCell<HashMap<BlockState, u32>>,
    block_lookup: HashMap<String, u32>,
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self {
            states: RefCell::new(Vec::new()),
            blocks: Vec::new(),
            id_lookup: RefCell::new(HashMap::new()),
            block_lookup: HashMap::new(),
        }
    }
}

impl BlockRegistry {
    /// Call this method only once per block.
    pub fn register_block<B: Block + 'static>(&mut self, block: B) -> BlockId {
        let index = self.blocks.len();
        self.block_lookup.insert(block.name().to_owned(), index as u32);
        self.blocks.push(Box::new(block));
        BlockId(index as u32)
    }

    pub fn register_state<B: Into<BlockState>>(&self, state: B) -> StateId {
        let state: BlockState = state.into();
        {
            let id_lookup = self.id_lookup.borrow();
            if let Some(&found) = id_lookup.get(&state) {
                return StateId(found);
            }
        }
        let block_id = *self.block_lookup.get(state.name()).expect("State not registered");
        let insert_index = {
            let states = self.states.borrow();
            states.len() as u32
        };
        let mut id_lookup = self.id_lookup.borrow_mut();
        id_lookup.insert(state.clone(), insert_index);
        let mut states = self.states.borrow_mut();
        states.push(RegistryEntry { block_id, state });
        StateId(insert_index)
    }

    pub fn find_state<B: Borrow<BlockState>>(&self, state: B) -> Option<StateId> {
        let id_lookup = self.id_lookup.borrow();
        id_lookup.get(state.borrow()).map(|&id| StateId(id))
    }

    pub fn find_block<S: AsRef<str>>(&self, name: S) -> Option<BlockId> {
        self.block_lookup.get(name.as_ref()).map(|&id| BlockId(id))
    }

    pub fn get_state(&self, id: StateId) -> Ref<BlockState> {
        let states = self.states.borrow();
        Ref::map(states, |v| &v[id.0 as usize].state)
        // &states[id.0 as usize].state
    }

    /// Find the Block that corresponds to the given state id. (Do not pass in block id, this method is for when you don't know the block id)
    pub fn get_block_for(&self, id: StateId) -> &dyn Block {
        let states = self.states.borrow();
        let block_id = states[id.0 as usize].block_id;
        self.blocks[block_id as usize].as_ref()
    }

    pub fn get_block(&self, id: BlockId) -> &dyn Block {
        self.blocks[id.0 as usize].as_ref()
    }

    pub fn get_block_cast<T: Block>(&self, id: BlockId) -> &T {
        let block: &dyn Any = &self.blocks[id.0 as usize];
        block.downcast_ref().expect("Failed to downcast")
    }

    pub fn get_block_cast_mut<T: Block>(&mut self, id: BlockId) -> &mut T {
        self.blocks[id.0 as usize].as_any_mut().downcast_mut().expect("Failed to downcast")
        // let block: &mut dyn Any = &mut self.blocks[id.0 as usize];
        // block.downcast_mut().expect("Failed to downcast")
    }

    pub fn get_state_and_block(&self, state_id: StateId) -> (Ref<BlockState>, &dyn Block) {
        let states = self.states.borrow();
        let block_id = states[state_id.0 as usize].block_id;
        let state = Ref::map(states, |states| &states[state_id.0 as usize].state);
        let block = self.blocks[block_id as usize].as_ref();
        (state, block)
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}