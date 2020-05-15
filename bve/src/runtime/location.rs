use crate::runtime::chunk::{ChunkAddress, ChunkOffset, CHUNK_SIZE};
use glam::Vec3;
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Location {
    pub chunk: ChunkAddress,
    pub offset: ChunkOffset,
}

impl Location {
    #[must_use]
    pub fn from_absolute_position(position: Vec3) -> Self {
        let x_chunk = (position.x() / CHUNK_SIZE).floor();
        let y_chunk = (position.z() / CHUNK_SIZE).floor();
        let chunk_start_position = Vec3::new(x_chunk * CHUNK_SIZE, 0.0, y_chunk * CHUNK_SIZE);
        Self {
            chunk: ChunkAddress::new(x_chunk as i32, y_chunk as i32),
            offset: (position - chunk_start_position).into(),
        }
    }

    #[must_use]
    pub const fn from_address_position(chunk: ChunkAddress, offset: ChunkOffset) -> Self {
        Self { chunk, offset }
    }

    #[must_use]
    pub fn to_relative_position(&self, base_chunk: ChunkAddress) -> Vec3 {
        let chunk_offset = *self.chunk - *base_chunk;
        Vec3::new(
            chunk_offset.x as f32 * CHUNK_SIZE,
            0.0,
            chunk_offset.y as f32 * CHUNK_SIZE,
        ) + *self.offset
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}):({}, {}, {})",
            self.chunk.x,
            self.chunk.y,
            self.offset.x(),
            self.offset.y(),
            self.offset.z()
        )
    }
}
