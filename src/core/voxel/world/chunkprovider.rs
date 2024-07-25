#![allow(unused)]
use crate::core::voxel::coord::Coord;

use super::{chunk::Chunk, chunkcoord::ChunkCoord};

pub trait ChunkProvider {
    fn load_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord.block_coord());
        self.load_into_chunk(coord, &mut chunk);
        chunk
    }

    fn load_into_chunk(&self, coord: ChunkCoord, chunk: &mut Chunk);

    fn save_chunk(&mut self, chunk: &Chunk);
}