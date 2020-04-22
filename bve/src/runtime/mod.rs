use crate::runtime::{
    cache::{MeshCache, PathSet, TextureCache},
    chunk::{Chunk, ChunkSet, ChunkState, UnloadedObject, CHUNK_SIZE},
};
pub use crate::runtime::{
    chunk::{ChunkAddress, ChunkOffset},
    client::Client,
    location::Location,
};
use async_std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    task::spawn,
};
use futures::{stream::FuturesOrdered, StreamExt};
use hecs::World;
use log::{debug, trace};
use std::sync::atomic::{AtomicI32, Ordering};

macro_rules! async_clone_own {
    ($($name:ident = $value:expr;)* { $($tokens:tt)* }) => {{
        $(
            let $name = $value.clone();
        )*
        async move {$(
            $tokens
        )*}
    }};
}

mod cache;
mod chunk;
mod client;
mod location;

#[allow(unused)]
struct ObjectTexture<C: Client> {
    object: C::ObjectHandle,
    mesh: C::MeshHandle,
    texture: C::TextureHandle,
}

#[allow(unused)]
pub struct Renderable<C: Client> {
    handles: Vec<ObjectTexture<C>>,
}

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

pub struct Runtime<C: Client> {
    client: Arc<Mutex<C>>,
    path_set: PathSet,
    chunks: ChunkSet,
    meshes: MeshCache<C>,
    textures: TextureCache<C>,
    position: Mutex<Location>,
    view_distance: AtomicI32,
    ecs: RwLock<World>,
}

impl<C: Client> Runtime<C> {
    #[must_use]
    pub fn new(client: Arc<Mutex<C>>) -> Arc<Self> {
        Arc::new(Self {
            client,
            path_set: PathSet::new(),
            chunks: ChunkSet::new(),
            meshes: MeshCache::new(),
            textures: TextureCache::new(),
            position: Mutex::new(Location {
                chunk: ChunkAddress::new(0, 0),
                offset: ChunkOffset::new(0.0, 0.0, 0.0),
            }),
            view_distance: AtomicI32::new((2048.0 / CHUNK_SIZE) as i32),
            ecs: RwLock::new(World::new()),
        })
    }

    // TODO: This probably should get refactored inside chunk
    pub async fn add_static_object(self: &Arc<Self>, location: Location, path: PathBuf) {
        trace!(
            "Adding object {} to chunk ({}, {}) at ({}, {}, {})",
            path.display(),
            location.chunk.x,
            location.chunk.y,
            location.offset.x,
            location.offset.y,
            location.offset.z,
        );
        let chunk = self.chunks.get_chunk(location.chunk).await;

        chunk.objects.insert(UnloadedObject {
            path: self
                .path_set
                .insert(path.canonicalize().await.expect("Could not canonicalize object"))
                .await,
            offset: location.offset,
        });
    }

    async fn load_mesh_textures(self: &Arc<Self>, path: PathBuf) -> Vec<(C::MeshHandle, C::TextureHandle)> {
        let mesh = self
            .meshes
            .load_mesh(&self.client, &self.path_set, path)
            .await
            .expect("Could not load mesh");

        let mut texture_futures = FuturesOrdered::new();
        for texture_path_handle in mesh.textures {
            texture_futures.push(spawn(async_clone_own!(runtime = self; { runtime.textures.load_texture_handle(&runtime.client, &runtime.path_set, texture_path_handle).await })));
        }

        let mut texture_handles = Vec::with_capacity(texture_futures.len());
        while let Some(option_texture_handle) = texture_futures.next().await {
            texture_handles.push(option_texture_handle.expect("TODO: Deal with unfound textures"));
        }

        let mut combined_handles = Vec::with_capacity(mesh.handles.len());
        for (mesh_handle, texture_idx) in mesh.handles {
            if let Some(texture_idx) = texture_idx {
                let texture_handle = texture_handles[texture_idx].clone();
                combined_handles.push((mesh_handle, texture_handle));
            } else {
                combined_handles.push((mesh_handle, C::TextureHandle::default()));
            }
        }

        combined_handles
    }

    async fn load_chunk_objects(self: &Arc<Self>, chunk: Arc<Chunk>) -> Vec<ObjectTexture<C>> {
        let mut mesh_futures = FuturesOrdered::new();
        let mut mesh_locations = Vec::new();
        for unloaded_object in chunk.objects.iter() {
            let path = self.path_set.get(unloaded_object.path).await;
            mesh_locations.push(unloaded_object.offset);
            mesh_futures.push(spawn(
                async_clone_own!(runtime = self; { runtime.load_mesh_textures(path).await }),
            ));
        }

        let mut object_textures = Vec::with_capacity(mesh_futures.len());

        let mut location_iter = mesh_locations.into_iter();
        while let (Some(mesh_handle_pairs), Some(location)) = (mesh_futures.next().await, location_iter.next()) {
            let mut client = self.client.lock().await;
            for (mesh_handle, texture_handle) in mesh_handle_pairs {
                let render_location = Location::from_address_position(chunk.address, location)
                    .to_relative_position(self.position.lock().await.chunk);

                let object_handle = client.add_object_texture(render_location, &mesh_handle, &texture_handle);
                object_textures.push(ObjectTexture {
                    mesh: mesh_handle,
                    texture: texture_handle,
                    object: object_handle,
                });
            }
        }

        object_textures
    }

    async fn load_chunk(self: Arc<Self>, chunk: Arc<Chunk>) {
        let handles = self.load_chunk_objects(Arc::clone(&chunk)).await;

        trace!("Adding chunk to ecs");
        let mut ecs = self.ecs.write().await;
        ecs.spawn((Renderable::<C> { handles },));
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

        for chunk_ref in self.chunks.inner.iter() {
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
