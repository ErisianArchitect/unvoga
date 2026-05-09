#![allow(unused)]
use std::{borrow::Borrow, ops::{Deref, Index}, sync::{Mutex, OnceLock}};

use bevy::math::Ray3d;
use hashbrown::HashMap;

use crate::{blockstate, core::voxel::blockstate};

use super::{block::Block, statevalue::StateValue, blockstate::BlockState, coord::Coord, lighting::lightargs::LightArgs, occluder::Occluder, occlusionshape::OcclusionShape, world::VoxelWorld};

struct RegistryEntry {
    state: BlockState,
    block_ref: BlockId,
}

struct Registry {
    states: Vec<&'static RegistryEntry>,
    blocks: Vec<&'static dyn Block>,
    id_lookup: HashMap<BlockState, Id>,
    block_lookup: HashMap<String, BlockId>,
}

static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| {
        let mut reg = Registry {
            states: Vec::with_capacity(4096),
            blocks: Vec::with_capacity(512),
            id_lookup: HashMap::new(),
            block_lookup: HashMap::new(),
        };
        Mutex::new(reg)
    })
}

fn init() {
    static DONE: OnceLock<()> = OnceLock::new();
    DONE.get_or_init(|| {
        register_block_raw(AirBlock);
        register_state_raw(blockstate!(air));
    });
}

fn register_block_raw<B: Block>(mut block: B) -> BlockId {
    let mut reg = registry().lock().unwrap();
    if reg.block_lookup.contains_key(block.name()) {
        panic!("Block already registered: {}", block.name());
    }
    let id = reg.blocks.len() as u32;
    reg.block_lookup.insert(block.name().to_owned(), BlockId(id));
    block.on_register();
    let leaked: &'static dyn Block = Box::leak(Box::new(block));
    reg.blocks.push(leaked);
    BlockId(id)
}

fn register_state_raw<B: Borrow<BlockState>>(state: B) -> Id {
    let mut reg = registry().lock().unwrap();
    if let Some(&id) = reg.id_lookup.get(state.borrow()) {
        return id;
    }
    let state: BlockState = state.borrow().clone();
    let block_id = if let Some(&block_id) = reg.block_lookup.get(state.name()) {
        block_id
    } else {
        panic!("Block not found: {}", state.name());
    };
    let id = reg.states.len() as u32;
    reg.id_lookup.insert(state.clone(), Id(id));
    let entry: &'static RegistryEntry = Box::leak(Box::new(RegistryEntry { block_ref: block_id, state }));
    reg.states.push(entry);
    Id(id)
}

#[must_use]
pub fn register_state<B: Borrow<BlockState>>(state: B) -> Id {
    init();
    register_state_raw(state)
}

#[must_use]
pub fn register_block<B: Block>(block: B) -> BlockId {
    init();
    register_block_raw(block)
}

/// If the [BlockState] has already been registered, find the associated [Id].
#[must_use]
pub fn find_state<B: Borrow<BlockState>>(state: B) -> Option<Id> {
    let reg = registry().lock().unwrap();
    reg.id_lookup.get(state.borrow()).copied()
}


#[must_use]
pub fn find_block<S: AsRef<str>>(name: S) -> Option<BlockId> {
    let reg = registry().lock().unwrap();
    reg.block_lookup.get(name.as_ref()).copied()
}


#[must_use]
pub fn get_block_ref(id: Id) -> BlockId {
    let reg = registry().lock().unwrap();
    reg.states[id.0 as usize].block_ref
}


#[must_use]
pub fn get_state(id: Id) -> &'static BlockState {
    let reg = registry().lock().unwrap();
    &reg.states[id.0 as usize].state
}


#[must_use]
pub fn get_block(id: BlockId) -> &'static dyn Block {
    let reg = registry().lock().unwrap();
    reg.blocks[id.0 as usize]
}


#[must_use]
pub fn get_block_for(id: Id) -> &'static dyn Block {
    let reg = registry().lock().unwrap();
    let block_id = reg.states[id.0 as usize].block_ref;
    reg.blocks[block_id.0 as usize]
}


#[must_use]
pub fn get_state_and_block(id: Id) -> (&'static BlockState, &'static dyn Block) {
    let reg = registry().lock().unwrap();
    let entry = reg.states[id.0 as usize];
    let block_id = entry.block_ref;
    let state: &'static BlockState = &entry.state;
    let block = reg.blocks[block_id.0 as usize];
    (state, block)
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
        get_state(*self)
    }
}

impl Deref for BlockId {
    type Target = dyn Block;

    fn deref(&self) -> &Self::Target {
        get_block(*self)
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