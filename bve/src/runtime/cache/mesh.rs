use crate::{
    filesystem::resolve_path,
    load::mesh::load_mesh_from_file,
    runtime::{
        cache::{PathHandle, PathSet},
        client::Client,
    },
};
use async_std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use dashmap::DashMap;
use log::trace;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct MeshData<C: Client> {
    pub handles: Vec<(C::MeshHandle, Option<usize>)>,
    pub textures: Vec<PathHandle>,
}

impl<C: Client> MeshData<C> {
    fn new() -> Self {
        Self {
            handles: Vec::new(),
            textures: Vec::new(),
        }
    }
}

impl<C: Client> Clone for MeshData<C> {
    fn clone(&self) -> Self {
        Self {
            handles: self.handles.clone(),
            textures: self.textures.clone(),
        }
    }
}

struct LoadedMesh<C: Client> {
    data: MeshData<C>,
    count: AtomicU64,
}

pub struct MeshCache<C: Client> {
    inner: DashMap<PathHandle, LoadedMesh<C>>,
}

impl<C: Client> MeshCache<C> {
    pub fn new() -> Self {
        Self { inner: DashMap::new() }
    }

    pub async fn load_mesh_impl(&self, client: &Mutex<C>, path_set: &PathSet, path: &Path) -> MeshData<C> {
        trace!("Loading mesh {}", path.display());
        let mesh = load_mesh_from_file(path)
            .await
            .expect("Path invalid, should have been validated earlier");

        let parent_dir = path.parent().expect("File must be in directory");

        let mut mesh_data = MeshData::<C>::new();
        for mesh in mesh.meshes {
            let handle = client.lock().await.add_mesh(mesh.vertices, &mesh.indices);
            mesh_data.handles.push((handle, mesh.texture.texture_id))
        }

        for texture in mesh.textures {
            let path = resolve_path(parent_dir, PathBuf::from(texture))
                .await
                .expect("Could not find texture");
            mesh_data.textures.push(path_set.insert(path).await);
        }

        mesh_data
    }

    pub async fn load_mesh(&self, client: &Mutex<C>, path_set: &PathSet, path: PathBuf) -> Option<MeshData<C>> {
        let canonicalized = path.canonicalize().await.ok()?;
        let path_handle = path_set.insert(canonicalized.clone()).await;
        Some(match self.inner.get(&path_handle) {
            Some(loaded) => {
                loaded.count.fetch_add(1, Ordering::AcqRel);
                loaded.data.clone()
            }
            None => {
                let mesh_data = self.load_mesh_impl(client, path_set, &canonicalized).await;
                self.inner.insert(path_handle, LoadedMesh {
                    data: mesh_data.clone(),
                    count: AtomicU64::new(1),
                });
                mesh_data
            }
        })
    }
}
