use crate::{
    runtime::{cache::PathHandle, LightDescriptor},
    IVec2,
};
use async_std::sync::{Arc, RwLock};
use dashmap::{DashMap, DashSet};
use derive_more::{AsMut, AsRef, Deref, Display, From as DmFrom, Into};
use glam::f32::Vec3;
use std::{
    hash::{Hash, Hasher},
    sync::atomic::AtomicU8,
};

pub const CHUNK_SIZE: f32 = 64.0;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deref, DmFrom, Into, Display, AsRef, AsMut)]
#[display(fmt = "({}, {})", "self.0.x", "self.0.y")]
pub struct ChunkAddress(IVec2);

impl ChunkAddress {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self(IVec2::new(x, y))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Deref, DmFrom, Into, Display, AsRef, AsMut)]
#[display(fmt = "({}, {}, {})", "self.0.x()", "self.0.y()", "self.0.z()")]
pub struct ChunkOffset(Vec3);

impl ChunkOffset {
    #[must_use]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
}

pub struct Chunk {
    pub address: ChunkAddress,
    pub objects: DashSet<UnloadedObject>,
    pub lights: RwLock<Vec<LightDescriptor>>,
    pub state: AtomicU8,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkState {
    Unloaded = 0,
    Loading = 1,
    Finished = 2,
    Unloading = 3,
}

impl From<u8> for ChunkState {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::Unloaded,
            1 => Self::Loading,
            2 => Self::Finished,
            3 => Self::Unloading,
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
        self.offset.x().to_bits().hash(state);
        self.offset.y().to_bits().hash(state);
        self.offset.z().to_bits().hash(state);
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
                    lights: RwLock::new(Vec::new()),
                    state: AtomicU8::new(ChunkState::Unloaded as u8),
                });
                self.inner.insert(address, Arc::clone(&arc));
                arc
            }
        }
    }
}
