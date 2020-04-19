use crate::runtime::chunk::{ChunkAddress, ChunkOffset};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Location {
    pub chunk: ChunkAddress,
    pub offset: ChunkOffset,
}
