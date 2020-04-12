use crate::{
    filesystem::resolve_path,
    load::mesh::{load_mesh_from_file, LoadedStaticMesh, Mesh, Texture, Vertex},
};
use async_std::{
    fs::read,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    task::spawn,
};
use cgmath::{Array, Vector2, Vector3};
use dashmap::{DashMap, DashSet};
use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use hecs::World;
use image::{guess_format, Rgba, RgbaImage};
use indexmap::set::IndexSet;
use itertools::Itertools;
use log::{debug, trace, warn};
use std::{
    hash::{Hash, Hasher},
    sync::atomic::{AtomicI32, AtomicU64, AtomicU8, Ordering},
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

pub trait Client: Send + Sync + 'static {
    type ObjectHandle: Clone + Hash + Send + Sync + 'static;
    type MeshHandle: Clone + Hash + Send + Sync + 'static;
    type TextureHandle: Clone + Hash + Send + Sync + 'static;

    fn add_object(&mut self, location: Vector3<f32>, mesh: &Self::MeshHandle, transparent: bool) -> Self::ObjectHandle;
    fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
        mesh: &Self::MeshHandle,
        texture: &Self::TextureHandle,
        transparent: bool,
    ) -> Self::ObjectHandle;
    fn add_mesh(&mut self, mesh_verts: Vec<Vertex>, indices: &[usize]) -> Self::MeshHandle;
    fn add_texture(&mut self, image: &RgbaImage) -> Self::TextureHandle;
}

const CHUNK_SIZE: f32 = 128.0;

pub type ChunkAddress = Vector2<i32>;
pub type ChunkOffset = Vector3<f32>;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Location {
    pub chunk: ChunkAddress,
    pub offset: ChunkOffset,
}

struct ObjectTexture<C: Client> {
    object: C::ObjectHandle,
    mesh: C::MeshHandle,
    texture: C::TextureHandle,
}
#[derive(Debug, Clone, PartialEq)]
struct UnloadedObject {
    path: PathHandle,
    offset: ChunkOffset,
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

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ChunkState {
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

struct Chunk {
    paths: DashSet<UnloadedObject>,
    state: AtomicU8,
}

struct ChunkComponent {
    address: ChunkAddress,
}

struct Renderable<C: Client> {
    handles: Vec<ObjectTexture<C>>,
}

pub fn is_mesh_transparent(mesh: &[Vertex]) -> bool {
    mesh.iter().any(|v| v.color.w != 0 && v.color.w != 255)
}

pub fn is_texture_transparent(texture: &RgbaImage) -> bool {
    texture.pixels().any(|&Rgba([_, _, _, a])| a != 0 && a != 255)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct PathHandle(usize);

struct LoadedMesh<C: Client> {
    handle: C::MeshHandle,
    count: AtomicU64,
}

struct LoadedTexture<C: Client> {
    handle: C::MeshHandle,
    count: AtomicU64,
}

pub struct Runtime<C: Client> {
    client: Arc<Mutex<C>>,
    path_cache: RwLock<IndexSet<PathBuf>>,
    chunks: DashMap<ChunkAddress, Arc<Chunk>>,
    meshes: DashMap<PathHandle, LoadedMesh<C>>,
    textures: DashMap<PathHandle, LoadedTexture<C>>,
    position: Mutex<Location>,
    view_distance: AtomicI32,
    ecs: RwLock<World>,
}

impl<C: Client> Runtime<C> {
    pub fn new(client: Arc<Mutex<C>>) -> Arc<Self> {
        Arc::new(Self {
            client,
            path_cache: RwLock::new(IndexSet::new()),
            chunks: DashMap::new(),
            meshes: DashMap::new(),
            textures: DashMap::new(),
            position: Mutex::new(Location {
                chunk: ChunkAddress::new(0, 0),
                offset: ChunkOffset::new(0.0, 0.0, 0.0),
            }),
            view_distance: AtomicI32::new((2048.0 / CHUNK_SIZE) as i32),
            ecs: RwLock::new(World::new()),
        })
    }

    async fn get_chunk(self: &Arc<Self>, address: ChunkAddress) -> Arc<Chunk> {
        match self.chunks.get(&address) {
            Some(e) => Arc::clone(e.value()),
            None => {
                let arc = Arc::new(Chunk {
                    paths: DashSet::new(),
                    state: AtomicU8::new(ChunkState::Unloaded as u8),
                });
                self.chunks.insert(address, Arc::clone(&arc));
                arc
            }
        }
    }

    async fn get_path(self: &Arc<Self>, path: PathHandle) -> PathBuf {
        self.path_cache
            .read()
            .await
            .get_index(path.0)
            .expect("Invalid path handle")
            .clone()
    }

    async fn register_path(self: &Arc<Self>, path: PathBuf) -> PathHandle {
        PathHandle(self.path_cache.write().await.insert_full(path).0)
    }

    pub async fn add_static_object(self: &Arc<Self>, location: Location, path: PathBuf) {
        let chunk = self.get_chunk(location.chunk).await;

        chunk.paths.insert(UnloadedObject {
            path: self.register_path(path).await,
            offset: location.offset,
        });
    }

    async fn load_single_texture(root_dir: PathBuf, relative: PathBuf) -> Option<RgbaImage> {
        let resolved_path = resolve_path(root_dir.clone(), relative.clone()).await;
        if let Some(path) = resolved_path {
            trace!("Loading texture {}", path.display());
            let data = read(path).await.expect("Cannot read file");
            let format = guess_format(&data).expect("Could not guess format");
            let image = image::load(std::io::Cursor::new(data), format).expect("Could not load image");
            Some(image.into_rgba())
        } else {
            warn!(
                "Could not find texture {} in {}",
                relative.display(),
                root_dir.display()
            );
            None
        }
    }

    async fn load_single_chunk_mesh(
        self: Arc<Self>,
        chunk: UnloadedObject,
    ) -> Option<(LoadedStaticMesh, Vec<RgbaImage>)> {
        let real_path = self.get_path(chunk.path).await;
        trace!("Loading mesh {}", real_path.display());
        let mesh_opt = load_mesh_from_file(&real_path).await;
        if let Some(mesh) = mesh_opt {
            trace!("Loaded mesh {}", real_path.display());
            let root_dir = real_path.parent().expect("File must have containing directory");
            let mut image_futures = FuturesOrdered::new();
            for texture in mesh.textures.iter() {
                let future = Self::load_single_texture(root_dir.to_path_buf(), PathBuf::from(texture));
                image_futures.push(spawn(future));
            }
            let mut images = Vec::with_capacity(mesh.textures.len());
            while let Some(image) = image_futures.next().await {
                images.push(if let Some(image) = image {
                    image
                } else {
                    RgbaImage::from_raw(1, 1, vec![0x00, 0xFF, 0xFF, 0xFF]).expect("Cannot create default image")
                })
            }
            Some((mesh, images))
        } else {
            warn!("Could not find mesh {}", real_path.display());
            None
        }
    }

    async fn load_chunk_objects(self: &Arc<Self>, chunk: Arc<Chunk>) -> Vec<(LoadedStaticMesh, Vec<RgbaImage>)> {
        let mut mesh_futures = FuturesUnordered::new();
        for mesh in chunk.paths.iter() {
            let mesh = mesh.clone();
            mesh_futures.push(spawn(Arc::clone(self).load_single_chunk_mesh(mesh)));
        }

        let mut meshes = Vec::with_capacity(mesh_futures.len());

        while let Some(maybe_mesh) = mesh_futures.next().await {
            if let Some(mesh) = maybe_mesh {
                meshes.push(mesh);
            }
        }

        meshes
    }

    fn unify_objects(input: Vec<(LoadedStaticMesh, Vec<RgbaImage>)>) -> (Vec<Mesh>, Vec<RgbaImage>) {
        let mut final_meshes = Vec::new();
        let mut final_textures =
            vec![RgbaImage::from_raw(1, 1, vec![0xFF, 0xFF, 0xFF, 0xFF]).expect("Cannot create default image")];
        for (objects, textures) in input {
            let texture_offset = final_textures.len();
            for texture in textures {
                final_textures.push(texture);
            }
            for mut mesh in objects.meshes {
                let id = &mut mesh.texture.texture_id;
                if let Some(id) = id {
                    *id += texture_offset;
                } else {
                    *id = Some(0);
                }
                final_meshes.push(mesh);
            }
        }

        (final_meshes, final_textures)
    }

    async fn load_chunk(self: Arc<Self>, chunk: Arc<Chunk>) {
        let objects = self.load_chunk_objects(Arc::clone(&chunk)).await;
        let (meshes, images) = Self::unify_objects(objects);

        trace!("Creating textures and objects in client");
        let mut client = self.client.lock().await;
        let texture_handles = images
            .into_iter()
            .map(|i| (is_texture_transparent(&i), client.add_texture(&i)))
            .collect_vec();
        let mut handles = Vec::new();
        for Mesh {
            vertices,
            indices,
            texture: Texture { texture_id, .. },
            ..
        } in meshes
        {
            let (tex_transparent, tex_handle) = texture_handles[texture_id.unwrap_or_else(|| unreachable!())].clone();
            let mesh_transparent = is_mesh_transparent(&vertices);
            let mesh_handle = client.add_mesh(vertices, &indices);
            handles.push(ObjectTexture::<C> {
                object: client.add_object_texture(
                    Vector3::from_value(0.0),
                    &mesh_handle,
                    &tex_handle,
                    tex_transparent | mesh_transparent,
                ),
                mesh: mesh_handle,
                texture: tex_handle,
            });
        }
        drop(client);

        trace!("Adding chunk to ecs");
        let mut ecs = self.ecs.write().await;
        ecs.spawn((Renderable::<C> { handles }, ChunkComponent {
            address: Vector2::new(0, 0),
        }));
        drop(ecs);

        chunk.state.store(ChunkState::Finished as u8, Ordering::Release);
        trace!("Chunk marked finished");
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

        for chunk_ref in self.chunks.iter() {
            let (&location, chunk) = chunk_ref.pair();
            let state = ChunkState::from(chunk.state.load(Ordering::Acquire));
            let inside = bounding_box.inside(location);
            if state == ChunkState::Finished && !inside {
                // deload
            } else if state == ChunkState::Unloaded && inside {
                debug!("Spawning chunk ({}, {})", location.x, location.y);
                let other_self = Arc::clone(self);
                spawn(other_self.load_chunk(Arc::clone(chunk)));
                chunk.state.store(ChunkState::Loading as u8, Ordering::Release);
            }
        }
    }
}
