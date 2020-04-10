use crate::{
    filesystem::resolve_path,
    load::mesh::{load_mesh_from_file, LoadedStaticMesh, Mesh, Vertex},
};
use async_std::{
    fs::read,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    task::spawn,
};
use cgmath::{Array, Deg, Matrix3, SquareMatrix, Vector2, Vector3};
use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use hecs::World;
use image::{guess_format, Rgba, RgbaImage};
use log::{debug, info, trace, warn};
use smallvec::{smallvec, SmallVec};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::atomic::{AtomicI32, AtomicU8, Ordering},
};
use texture_packer::{exporter::ImageExporter, TexturePacker, TexturePackerConfig};

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
    type ObjectHandle: Send + Sync + 'static;
    type TextureHandle: Send + Sync + 'static;

    fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
        verts: Vec<Vertex>,
        indices: &[usize],
        texture: &Self::TextureHandle,
    ) -> Self::ObjectHandle;
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
    texture: C::TextureHandle,
}
#[derive(Debug, Clone, PartialEq)]
struct UnloadedObject {
    path: PathBuf,
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
    paths: RwLock<HashSet<UnloadedObject>>,
    state: AtomicU8,
}

struct ChunkComponent {
    address: ChunkAddress,
}

struct Renderable<C: Client> {
    handles: SmallVec<[ObjectTexture<C>; 4]>,
}

pub fn is_mesh_transparent(mesh: &[Vertex]) -> bool {
    mesh.iter().any(|v| v.color.w != 0.0 && v.color.w != 1.0)
}

pub fn is_texture_transparent(texture: &RgbaImage) -> bool {
    texture.pixels().any(|&Rgba([_, _, _, a])| a != 0 && a != 255)
}

pub struct Runtime<C: Client> {
    client: Arc<Mutex<C>>,
    chunks: RwLock<HashMap<ChunkAddress, Arc<Chunk>>>,
    position: Mutex<Location>,
    view_distance: AtomicI32,
    ecs: RwLock<World>,
}

impl<C: Client> Runtime<C> {
    pub fn new(client: Arc<Mutex<C>>) -> Arc<Self> {
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
                    state: AtomicU8::new(ChunkState::Unloaded as u8),
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

    async fn load_single_chunk_mesh(chunk: UnloadedObject) -> Option<(LoadedStaticMesh, Vec<RgbaImage>)> {
        trace!("Loading mesh {}", chunk.path.display());
        let mesh_opt = load_mesh_from_file(&chunk.path).await;
        if let Some(mesh) = mesh_opt {
            trace!("Loaded mesh {}", chunk.path.display());
            let root_dir = chunk.path.parent().expect("File must have containing directory");
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
            warn!("Could not find mesh {}", chunk.path.display());
            None
        }
    }

    async fn load_chunk_objects(chunk: Arc<Chunk>) -> Vec<(LoadedStaticMesh, Vec<RgbaImage>)> {
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

    fn create_packed_textures(images: Vec<RgbaImage>) -> (RgbaImage, Vec<Matrix3<f32>>) {
        let mut packer = TexturePacker::new_skyline(TexturePackerConfig {
            max_width: 1 << 14,
            max_height: 1 << 14,
            texture_padding: 32,
            border_padding: 32,
            allow_rotation: true,
            texture_outlines: false,
            trim: false,
        });

        let image_count = images.len();

        for (idx, image) in images.into_iter().enumerate() {
            packer.pack_own(idx.to_string(), image).expect("Packing failure");
        }

        let texture_atlas = ImageExporter::export(&packer)
            .expect("Unable to export texture atlas")
            .into_rgba();

        let max_width = (texture_atlas.width() - 1) as f32;
        let max_height = (texture_atlas.height() - 1) as f32;

        let mut transforms = vec![Matrix3::identity(); image_count];

        for (string, frame) in packer.get_frames() {
            let idx: usize = string.parse().expect("Unable to parse");
            let inner = &frame.frame;
            let translation =
                Matrix3::from_translation(Vector2::new(inner.x as f32 / max_width, inner.y as f32 / max_height));
            let scale =
                Matrix3::from_nonuniform_scale((inner.w - 1) as f32 / max_width, (inner.h - 1) as f32 / max_height);
            if frame.rotated {
                let rot_trans = Matrix3::from_translation(Vector2::new(1.0, 0.0));
                let rot_rot = Matrix3::from_angle_z(Deg(90.0));
                transforms[idx] = translation * scale * rot_trans * rot_rot;
            } else {
                transforms[idx] = translation * scale;
            }
        }

        (texture_atlas, transforms)
    }

    async fn load_chunk(self: Arc<Self>, chunk: Arc<Chunk>) {
        let objects = Self::load_chunk_objects(Arc::clone(&chunk)).await;
        let (meshes, images) = Self::unify_objects(objects);

        info!("Preforming texture packing on chunk ({}, {})", 0, 0);
        let (texture_atlas, transforms) = Self::create_packed_textures(images);
        info!("Texture packing finished on chunk ({}, {})", 0, 0);

        let mut atlas_verts: Vec<Vertex> = Vec::new();
        let mut atlas_indices: Vec<usize> = Vec::new();

        for mut mesh in meshes {
            let vert_offset = atlas_verts.len();
            let texture_id = mesh.texture.texture_id.unwrap_or_else(|| unreachable!());
            mesh.vertices
                .iter_mut()
                .for_each(|v| v.coord_transform = transforms[texture_id]);
            atlas_verts.extend(mesh.vertices.into_iter());
            atlas_indices.extend(mesh.indices.into_iter().map(|i| i + vert_offset));
        }

        trace!("Creating texture and object in client");
        let mut client = self.client.lock().await;
        let texture = client.add_texture(&texture_atlas);
        let object = client.add_object_texture(Vector3::from_value(0.0), atlas_verts, &atlas_indices, &texture);
        drop(client);

        let handles = smallvec![ObjectTexture { object, texture }];

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

        for (&location, chunk) in self.chunks.read().await.iter() {
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
