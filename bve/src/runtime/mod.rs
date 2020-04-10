use crate::{
    filesystem::resolve_path,
    load::mesh::{load_mesh_from_file, LoadedStaticMesh, Vertex},
};
use async_std::{
    fs::read,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    task::spawn,
};
use cgmath::{Vector2, Vector3};
use futures::{stream::FuturesUnordered, StreamExt};
use hecs::World;
use image::{guess_format, ImageFormat, RgbaImage};
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
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

#[derive(Debug, Clone, PartialEq)]
struct UnloadedObject {
    path: PathBuf,
    offset: ChunkOffset,
}

impl Eq for UnloadedObject {}

impl Hash for UnloadedObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.offset.x.to_bits().hash(state);
        self.offset.y.to_bits().hash(state);
        self.offset.z.to_bits().hash(state);
    }
}

struct Chunk {
    paths: RwLock<HashSet<UnloadedObject>>,
    loaded: AtomicBool,
}

struct ChunkComponent {
    address: Location,
}

struct Renderable<C: Client> {
    handles: SmallVec<[ObjectTexture<C>; 4]>,
}

pub struct Runtime<C: Client + Send + Sync + 'static> {
    client: Arc<C>,
    chunks: RwLock<HashMap<ChunkAddress, Arc<Chunk>>>,
    position: Mutex<Location>,
    view_distance: AtomicI32,
    ecs: RwLock<World>,
}

impl<C: Client + Send + Sync + 'static> Runtime<C> {
    pub fn new(client: Arc<C>) -> Arc<Self> {
        Arc::new(Self {
            client,
            chunks: RwLock::new(HashMap::new()),
            position: Mutex::new(Location {
                chunk: ChunkAddress::new(0, 0),
                offset: ChunkOffset::new(0.0, 0.0, 0.0),
            }),
            view_distance: AtomicI32::new((2048.0 / CHUNK_SIZE) as i32),
            ecs: RwLock::new(World::new()),
        })
    }

    async fn get_chunk(self: &Arc<Self>, address: ChunkAddress) -> Arc<Chunk> {
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

    pub async fn add_static_object(self: &Arc<Self>, location: Location, path: PathBuf) {
        let chunk = self.get_chunk(location.chunk).await;

        let mut paths = chunk.paths.write().await;
        paths.insert(UnloadedObject {
            path,
            offset: location.offset,
        });
    }

    async fn load_single_texture(root_dir: PathBuf, relative: PathBuf) -> Option<RgbaImage> {
        let resolved_path = resolve_path(root_dir, relative).await;
        if let Some(path) = resolved_path {
            let data = read(path).await.expect("Cannot read file");
            let format = guess_format(&data).expect("Could not guess format");
            let image = image::load(std::io::Cursor::new(data), format).expect("Could not load image");
            Some(image.into_rgba())
        } else {
            None
        }
    }

    async fn load_single_chunk_mesh(chunk: UnloadedObject) -> Option<(LoadedStaticMesh, Vec<RgbaImage>)> {
        let mesh_opt = load_mesh_from_file(&chunk.path).await;
        if let Some(mesh) = mesh_opt {
            let root_dir = chunk.path.parent().expect("File must have containing directory");
            let mut image_futures = FuturesUnordered::new();
            for texture in mesh.textures.iter() {
                let future = Self::load_single_texture(root_dir.to_path_buf(), PathBuf::from(texture));
                image_futures.push(spawn(future));
            }
            let mut images = Vec::with_capacity(mesh.textures.len());
            for image in image_futures.next().await {
                if let Some(image) = image {
                    images.push(image);
                }
            }
            Some((mesh, images))
        } else {
            None
        }
    }

    async fn load_chunk_meshes(chunk: Arc<Chunk>) -> Vec<(LoadedStaticMesh, Vec<RgbaImage>)> {
        let mesh_list = chunk.paths.read().await;
        let mut mesh_futures = FuturesUnordered::new();
        for mesh in mesh_list.iter() {
            let mesh = mesh.clone();
            mesh_futures.push(spawn(Self::load_single_chunk_mesh(mesh)));
        }

        let mut meshes = Vec::with_capacity(mesh_futures.len());

        while let Some(maybe_mesh) = mesh_futures.next().await {
            if let Some(mesh) = maybe_mesh {
                meshes.push(mesh);
            }
        }

        meshes
    }

    async fn load_chunk(self: Arc<Self>, chunk: Arc<Chunk>) {
        let meshes = Self::load_chunk_meshes(chunk).await;
    }

    pub async fn tick(self: &Arc<Self>) {
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
                let other_self = Arc::clone(self);
                spawn(other_self.load_chunk(Arc::clone(chunk)));
            }
        }
    }
}
