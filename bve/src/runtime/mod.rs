use crate::load::mesh::Vertex;
use async_std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use cgmath::{Vector2, Vector3};
use std::collections::{HashMap, HashSet};

pub trait Client {
    type ObjectHandle;

    fn add_object(&self, location: Vector3<f32>, verts: &[Vertex], indices: &[usize]) -> Self::ObjectHandle;
}

type ChunkAddress = Vector2<i32>;
type ChunkOffset = Vector3<f32>;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Location {
    pub chunk: ChunkAddress,
    pub offset: ChunkOffset,
}

struct Chunk {
    paths: RwLock<HashSet<PathBuf>>,
    loaded: RwLock<bool>,
}

pub struct Runtime<C: Client> {
    client: Arc<C>,
    chunks: RwLock<HashMap<ChunkAddress, Arc<Chunk>>>,
    position: Mutex<Location>,
}

impl<C: Client> Runtime<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            chunks: RwLock::new(HashMap::new()),
            position: Mutex::new(Location {
                chunk: ChunkAddress::new(0, 0),
                offset: ChunkOffset::new(0.0, 0.0, 0.0),
            }),
        }
    }

    async fn get_chunk(&self, location: Location) -> Arc<Chunk> {
        let chunk_map = self.chunks.read().await;
        match chunk_map.get(&location.chunk) {
            Some(e) => Arc::clone(e),
            None => {
                drop(chunk_map);
                let mut chunk_map_mut = self.chunks.write().await;
                let arc = Arc::new(Chunk {
                    paths: RwLock::new(HashSet::new()),
                    loaded: RwLock::new(false),
                });
                chunk_map_mut.insert(location.chunk, Arc::clone(&arc));
                arc
            }
        }
    }

    pub async fn add_static_object(&self, location: Location, path: PathBuf) {
        let chunk = self.get_chunk(location).await;

        let mut paths = chunk.paths.write().await;
        paths.insert(path);
    }
}
