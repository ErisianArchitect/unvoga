use rollgrid::rollgrid2d::Bounds2D;

use crate::{core::voxel::world::{VoxelWorld, WORLD_BOTTOM, WORLD_TOP}, prelude::Id};

pub trait WorldGenerator: Send + Sync {
    fn generate_chunk(&mut self, world: &mut VoxelWorld, area: Bounds2D) {}
}

impl WorldGenerator for () {}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct FlatLayer {
    height: u16,
    block: Id,
}

impl FlatLayer {
    pub const fn new(height: u16, block: Id) -> Self {
        Self {
            height,
            block,
        }
    }
}

impl From<(u16, Id)> for FlatLayer {
    fn from(value: (u16, Id)) -> Self {
        Self::new(value.0, value.1)
    }
}

pub struct FlatWorldGenerator {
    layers: Vec<FlatLayer>,
}

impl FlatWorldGenerator {
    pub fn new(layers: Vec<FlatLayer>) -> Self {
        Self {
            layers
        }
    }
}

impl<L: Into<FlatLayer>> FromIterator<L> for FlatWorldGenerator {
    fn from_iter<T: IntoIterator<Item = L>>(iter: T) -> Self {
        Self {
            layers: iter.into_iter().map(L::into).collect(),
        }
    }
}

impl WorldGenerator for FlatWorldGenerator {
    fn generate_chunk(&mut self, world: &mut VoxelWorld, area: Bounds2D) {
        for (x, z) in area.iter() {
            let mut y_bottom = WORLD_BOTTOM;
            for layer in self.layers.iter() {
                let y_top = (y_bottom + layer.height as i32).min(WORLD_TOP);
                for y in y_bottom..y_top {
                    world.set_block((x, y, z), layer.block);
                }
                y_bottom = y_top;
            }
        }
    }
}