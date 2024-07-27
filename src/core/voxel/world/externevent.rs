use crate::prelude::Coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternEvent {
    /// (section coord)
    RenderChunkMoved(Coord),
}