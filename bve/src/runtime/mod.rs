use crate::load::mesh::Vertex;
use async_std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use cgmath::{Vector2, Vector3};
use hecs::World;
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicBool, AtomicI32, Ordering},
};

struct BoundingBox {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

impl BoundingBox {
    fn inside(&self, point: ChunkAddress) -> bool {
        self.min_x <= point.x && point.x <= self.max_x && self.min_y <= point.y && point.y <= self.max_y
    }
}

pub trait Client {
    type ObjectHandle;
    type TextureHandle;

    fn add_object(&self, location: Vector3<f32>, verts: &[Vertex], indices: &[usize]) -> Self::ObjectHandle;
}

const CHUNK_SIZE: f32 = 128.0;

type ChunkAddress = Vector2<i32>;
type ChunkOffset = Vector3<f32>;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Location {
    pub chunk: ChunkAddress,
    pub offset: ChunkOffset,
}

struct ObjectTexture<C: Client> {
    object: C::ObjectHandle,
    texture: C::TextureHandle,
}

struct Chunk {
    paths: RwLock<HashSet<PathBuf>>,
    loaded: AtomicBool,
}

struct ChunkComponent {
    address: Location,
}

struct Renderable<C: Client> {
    handles: SmallVec<[ObjectTexture<C>; 4]>,
}

pub struct Runtime<C: Client> {
    client: Arc<C>,
    chunks: RwLock<HashMap<ChunkAddress, Arc<Chunk>>>,
    position: Mutex<Location>,
    view_distance: AtomicI32,
    ecs: RwLock<World>,
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
            view_distance: AtomicI32::new((2048.0 / CHUNK_SIZE) as i32),
            ecs: RwLock::new(World::new()),
        }
    }

    async fn get_chunk(&self, address: ChunkAddress) -> Arc<Chunk> {
        let chunk_map = self.chunks.read().await;
        match chunk_map.get(&address) {
            Some(e) => Arc::clone(e),
            None => {
                drop(chunk_map);
                let mut chunk_map_mut = self.chunks.write().await;
                let arc = Arc::new(Chunk {
                    paths: RwLock::new(HashSet::new()),
                    loaded: AtomicBool::new(false),
                });
                chunk_map_mut.insert(address, Arc::clone(&arc));
                arc
            }
        }
    }

    pub async fn add_static_object(&self, location: Location, path: PathBuf) {
        let chunk = self.get_chunk(location.chunk).await;

        let mut paths = chunk.paths.write().await;
        paths.insert(path);
    }

    pub async fn tick(&self) {
        let view_distance = self.view_distance.load(Ordering::Relaxed);
        let location = self.position.lock().await.chunk;
        let bounding_box = BoundingBox {
            min_x: location.x - view_distance,
            max_x: location.x + view_distance,
            min_y: location.y - view_distance,
            max_y: location.y + view_distance,
        };

        for (&location, chunk) in self.chunks.read().await.iter() {
            let loaded = chunk.loaded.load(Ordering::Acquire);
            let inside = bounding_box.inside(location);
            if loaded && !inside {
                // deload
            } else if !loaded && inside {
                // load
            }
        }
    }
}
