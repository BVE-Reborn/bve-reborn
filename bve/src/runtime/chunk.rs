use crate::runtime::cache::PathHandle;
use async_std::sync::Arc;
use cgmath::{Vector2, Vector3};
use dashmap::{DashMap, DashSet};
use std::{
    hash::{Hash, Hasher},
    sync::atomic::AtomicU8,
};

pub const CHUNK_SIZE: f32 = 128.0;

pub type ChunkAddress = Vector2<i32>;
pub type ChunkOffset = Vector3<f32>;

pub struct Chunk {
    pub address: ChunkAddress,
    pub objects: DashSet<UnloadedObject>,
    pub state: AtomicU8,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkState {
    Unloaded = 0,
    Loading = 1,
    Finished = 2,
}

impl From<u8> for ChunkState {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::Unloaded,
            1 => Self::Loading,
            2 => Self::Finished,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnloadedObject {
    pub path: PathHandle,
    pub offset: ChunkOffset,
}

impl Eq for UnloadedObject {}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for UnloadedObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.offset.x.to_bits().hash(state);
        self.offset.y.to_bits().hash(state);
        self.offset.z.to_bits().hash(state);
    }
}

pub struct ChunkSet {
    pub inner: DashMap<ChunkAddress, Arc<Chunk>>,
}

impl ChunkSet {
    pub fn new() -> Self {
        Self { inner: DashMap::new() }
    }

    pub async fn get_chunk(&self, address: ChunkAddress) -> Arc<Chunk> {
        match self.inner.get(&address) {
            Some(e) => Arc::clone(e.value()),
            None => {
                let arc = Arc::new(Chunk {
                    address,
                    objects: DashSet::new(),
                    state: AtomicU8::new(ChunkState::Unloaded as u8),
                });
                self.inner.insert(address, Arc::clone(&arc));
                arc
            }
        }
    }
}
