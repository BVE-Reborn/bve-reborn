pub use crate::runtime::{
    chunk::{ChunkAddress, ChunkOffset},
    client::Client,
    executor::*,
    light::*,
    location::Location,
};
use crate::{
    runtime::{
        cache::{MeshCache, PathHandle, PathSet, TextureCache},
        chunk::{Chunk, ChunkSet, ChunkState, UnloadedObject, CHUNK_SIZE},
    },
    AsyncMutex, AsyncRwLock,
};
use async_std::path::PathBuf;
use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use hecs::World;
use log::{debug, trace};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

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
mod executor;
mod light;
mod location;

struct RenderableObject<C: Client> {
    object: C::ObjectHandle,
    location: Location,
}

struct RenderableComponent<C: Client> {
    subobjects: Vec<RenderableObject<C>>,
}

struct ChunkComponent {
    address: ChunkAddress,
}

struct Light<C: Client> {
    handle: C::LightHandle,
    descriptor: LightDescriptor,
}

struct LightOwnerComponent<C: Client> {
    lights: Vec<Light<C>>,
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

#[derive(Debug, Copy, Clone)]
struct RuntimeLocation {
    location: Location,
    old_location: Location,
}

const DEFAULT_RENDER_DISTANCE: f32 = 32.0 * CHUNK_SIZE;

// Mutexes are always grabbed in the following order
// ecs -> chunks -> client -> location
pub struct Runtime<C: Client> {
    client: Arc<AsyncMutex<C>>,
    path_set: PathSet,
    chunks: ChunkSet,
    meshes: MeshCache<C>,
    textures: TextureCache<C>,
    location: AsyncMutex<RuntimeLocation>,
    view_distance: AtomicI32,
    ecs: AsyncRwLock<World>,
}

impl<C: Client> Runtime<C> {
    #[must_use]
    pub fn new(client: Arc<AsyncMutex<C>>) -> Arc<Self> {
        Arc::new(Self {
            client,
            path_set: PathSet::new(),
            chunks: ChunkSet::new(),
            meshes: MeshCache::new(),
            textures: TextureCache::new(),
            location: AsyncMutex::new(
                RuntimeLocation {
                    location: Location {
                        chunk: ChunkAddress::new(0, 0),
                        offset: ChunkOffset::new(0.0, 0.0, 0.0),
                    },
                    old_location: Location {
                        chunk: ChunkAddress::new(0, 0),
                        offset: ChunkOffset::new(0.0, 0.0, 0.0),
                    },
                },
                false,
            ),
            view_distance: AtomicI32::new((DEFAULT_RENDER_DISTANCE / CHUNK_SIZE) as i32),
            ecs: AsyncRwLock::new(World::new()),
        })
    }

    // TODO: This probably should get refactored inside chunk
    pub async fn add_static_object(self: &Arc<Self>, location: Location, path: PathBuf) {
        trace!("Adding object {} to chunk {}", path.display(), location);
        let chunk = self.chunks.get_chunk(location.chunk).await;

        chunk.objects.insert(UnloadedObject {
            path: self
                .path_set
                .insert(path.canonicalize().await.expect("Could not canonicalize object"))
                .await,
            offset: location.offset,
        });
    }

    pub async fn add_light(self: &Arc<Self>, light_descriptor: LightDescriptor) {
        trace!("Adding light at position {}", light_descriptor.location);
        let chunk = self.chunks.get_chunk(light_descriptor.location.chunk).await;

        chunk.lights.write().await.push(light_descriptor);
    }

    async fn load_mesh_textures(self: &Arc<Self>, path: PathBuf) -> Vec<(C::MeshHandle, C::TextureHandle)> {
        let mesh = self
            .meshes
            .load_mesh(&self.client, &self.path_set, path)
            .await
            .expect("Could not load mesh");

        let mut texture_futures = FuturesOrdered::new();
        for texture_path_handle in mesh.textures {
            texture_futures.push(spawn(Pool::IO, 0, async_clone_own!(runtime = self; { runtime.textures.load_texture_handle(&runtime.client, &runtime.path_set, texture_path_handle).await })));
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

    async fn load_chunk_objects(self: Arc<Self>, chunk: Arc<Chunk>) -> Vec<RenderableObject<C>> {
        let mut mesh_futures = FuturesOrdered::new();
        let mut mesh_locations = Vec::new();
        for unloaded_object in chunk.objects.iter() {
            let path = self.path_set.get(unloaded_object.path);
            mesh_locations.push(unloaded_object.offset);
            mesh_futures.push(spawn(
                Pool::IO,
                0,
                async_clone_own!(runtime = self; { runtime.load_mesh_textures(path).await }),
            ));
        }

        let mut object_textures = Vec::with_capacity(mesh_futures.len());

        let mut location_iter = mesh_locations.into_iter();
        while let (Some(mesh_handle_pairs), Some(chunk_offset)) = (mesh_futures.next().await, location_iter.next()) {
            let mut client = self.client.lock().await;
            // Because we have a lock on the client, we can't be in the middle of a location update operation
            // so there is no need to keep accessing this data. We can just grab it and move on.
            let base_chunk = self.location.lock().await.location.chunk;
            for (mesh_handle, texture_handle) in mesh_handle_pairs {
                let location = Location::from_address_position(chunk.address, chunk_offset);
                let render_location = location.to_relative_position(base_chunk);

                let object_handle = client.add_object_texture(render_location, &mesh_handle, &texture_handle);
                object_textures.push(RenderableObject {
                    object: object_handle,
                    location,
                });
            }
        }

        object_textures
    }

    async fn load_chunk_lights(self: Arc<Self>, chunk: Arc<Chunk>) -> Vec<Light<C>> {
        let lights = chunk.lights.read().await;
        let mut client = self.client.lock().await;
        let base_chunk = self.location.lock().await.location.chunk;

        let mut res = Vec::with_capacity(lights.len());
        for light in &*lights {
            let handle = client.add_light(light.into_render_light_descriptor(base_chunk));
            res.push(Light {
                handle,
                descriptor: *light,
            });
        }
        res
    }

    async fn load_chunk(self: Arc<Self>, chunk: Arc<Chunk>) {
        let subobjects_handle = spawn(
            Pool::IO,
            0,
            async_clone_own!(runtime = self; chunk = chunk; { runtime.load_chunk_objects(chunk).await }),
        );
        let lights_handle = spawn(
            Pool::IO,
            0,
            async_clone_own!(runtime = self; chunk = chunk; { runtime.load_chunk_lights(chunk).await }),
        );

        let subobjects = subobjects_handle.await;
        let lights = lights_handle.await;
        trace!("Adding chunk to ecs");
        let mut ecs = self.ecs.write().await;
        ecs.spawn((
            RenderableComponent::<C> { subobjects },
            ChunkComponent { address: chunk.address },
            LightOwnerComponent { lights },
        ));
        drop(ecs);

        chunk.state.store(ChunkState::Finished as u8, Ordering::Release);
        trace!("Chunk marked finished");
    }

    async fn deload_mesh(self: Arc<Self>, path_handle: PathHandle) {
        let textures_opt = self.meshes.remove_mesh(&self.client, path_handle).await;
        if let Some(textures) = textures_opt {
            for texture_handle in textures {
                // This could be a separate task, but I don't think this will really help things, there's
                // no major operations going on here, just locks being grabbed
                self.textures.remove_texture(&self.client, texture_handle).await;
            }
        }
    }

    async fn deload_chunk(self: Arc<Self>, chunk: Arc<Chunk>) {
        let mut ecs = self.ecs.write().await;
        let mut query = ecs.query::<(&RenderableComponent<C>, &LightOwnerComponent<C>, &ChunkComponent)>();
        let mut iter = query.iter().filter(|(_, (_, _, c))| c.address == chunk.address);
        if let Some((id, (renderable, light_owner, _))) = iter.next() {
            let mut client = self.client.lock().await;
            let renderable: &RenderableComponent<C> = renderable;
            for subobject in &renderable.subobjects {
                client.remove_object(&subobject.object);
            }
            let light_owner: &LightOwnerComponent<C> = light_owner;
            for light in &light_owner.lights {
                client.remove_light(&light.handle);
            }
            drop(query);
            ecs.despawn(id).expect("Could not find entity");
        } else {
            drop(query);
        }
        drop(ecs);

        let mut despawn_futures = FuturesUnordered::new();
        for subobject in chunk.objects.iter() {
            let mesh_path = subobject.path;
            despawn_futures.push(spawn(
                Pool::IO,
                0,
                async_clone_own!(runtime = self; { runtime.deload_mesh(mesh_path).await }),
            ));
        }

        while let Some(..) = despawn_futures.next().await {
            // empty
        }

        // Everything is deloaded, lets mark it so
        chunk.state.store(ChunkState::Unloaded as u8, Ordering::Release);
    }

    pub async fn set_location(&self, location: Location) {
        let mut runtime_location = self.location.lock().await;
        runtime_location.location = location;
        drop(runtime_location);
        let mut client = self.client.lock().await;
        client.set_camera_location(*location.offset);
    }

    async fn update_camera_position(self: Arc<Self>, base_location: Location) {
        trace!("Updating camera to location: {}", base_location);
        let ecs = self.ecs.read().await;
        let mut client = self.client.lock().await;
        for (_id, (renderable,)) in ecs.query::<(&RenderableComponent<C>,)>().iter() {
            let renderable: &RenderableComponent<C> = renderable;
            for object in &renderable.subobjects {
                let render_location = object.location.to_relative_position(base_location.chunk);
                client.set_object_location(&object.object, render_location);
            }
        }
        for (_id, (light_owner,)) in ecs.query::<(&LightOwnerComponent<C>,)>().iter() {
            let light_owner: &LightOwnerComponent<C> = light_owner;
            for light in &light_owner.lights {
                let light: &Light<C> = light;
                let render_descriptor = light.descriptor.into_render_light_descriptor(base_location.chunk);
                client.set_light_descriptor(&light.handle, render_descriptor);
            }
        }
    }

    pub async fn tick(self: &Arc<Self>) {
        let view_distance = self.view_distance.load(Ordering::Relaxed);

        // Handle runtime location, and spawn off job to update positions if needed
        let mut runtime_location = self.location.lock().await;
        let location = runtime_location.location;
        #[allow(clippy::if_not_else)]
        let location_update_job = if runtime_location.location.chunk != runtime_location.old_location.chunk {
            runtime_location.old_location = runtime_location.location;
            drop(runtime_location);
            // We're no longer in the same chunk, so we need to update the positions of objects
            Some(spawn(
                Pool::IO,
                0,
                async_clone_own!(runtime = self; {runtime.update_camera_position(location).await}),
            ))
        } else {
            drop(runtime_location);
            None
        };
        // self.location lock must not survive beyond this point

        let bounding_box = BoundingBox {
            min_x: location.chunk.x - view_distance,
            max_x: location.chunk.x + view_distance,
            min_y: location.chunk.y - view_distance,
            max_y: location.chunk.y + view_distance,
        };

        for chunk_ref in self.chunks.inner.iter() {
            let (&location, chunk) = chunk_ref.pair();
            let state = ChunkState::from(chunk.state.load(Ordering::Acquire));
            let inside = bounding_box.inside(location);
            if state == ChunkState::Finished && !inside {
                debug!("Deloading chunk ({}, {})", location.x, location.y);
                spawn(
                    Pool::IO,
                    0,
                    async_clone_own!(runtime = self; chunk = chunk; { runtime.deload_chunk(chunk).await }),
                );
                chunk.state.store(ChunkState::Unloading as u8, Ordering::Release);
            } else if state == ChunkState::Unloaded && inside {
                debug!("Loading chunk ({}, {})", location.x, location.y);
                spawn(
                    Pool::IO,
                    0,
                    async_clone_own!(runtime = self; chunk = chunk; { runtime.load_chunk(chunk).await }),
                );
                chunk.state.store(ChunkState::Loading as u8, Ordering::Release);
            }
        }

        if let Some(join_handle) = location_update_job {
            join_handle.await;
        }
    }
}
