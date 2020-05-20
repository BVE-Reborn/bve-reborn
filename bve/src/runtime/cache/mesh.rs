use crate::{
    filesystem::resolve_path,
    load::mesh::{load_mesh_from_file, LoadedStaticMesh, Vertex},
    runtime::{
        cache::{Cache, PathHandle, PathSet},
        client::Client,
    },
};
use async_std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use itertools::Itertools;
use log::trace;

struct RawMeshData {
    meshes: Vec<(Vec<Vertex>, Vec<usize>, Option<usize>)>,
    textures: Vec<String>,
}

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

pub struct MeshCache<C: Client> {
    inner: Cache<MeshData<C>>,
}

impl<C: Client> MeshCache<C> {
    pub fn new() -> Self {
        Self { inner: Cache::new() }
    }

    fn combine_eligible_meshes(mut meta_mesh: LoadedStaticMesh) -> RawMeshData {
        meta_mesh.meshes.sort_by_key(|m| m.texture.texture_id);
        let mut meshes = Vec::new();
        for (texture_id, group) in &meta_mesh.meshes.into_iter().group_by(|m| m.texture.texture_id) {
            let mut verts = Vec::new();
            let mut indices = Vec::new();
            for mesh in group {
                let vert_offset = verts.len();
                verts.extend(mesh.vertices.into_iter());
                indices.extend(mesh.indices.into_iter().map(|i| i + vert_offset));
            }
            meshes.push((verts, indices, texture_id));
        }

        RawMeshData {
            meshes,
            textures: meta_mesh.textures.into_iter().collect(),
        }
    }

    async fn load_mesh_impl(&self, client: &Mutex<C>, path_set: &PathSet, path: &Path) -> MeshData<C> {
        trace!("Loading mesh {}", path.display());
        let meta_mesh = load_mesh_from_file(path)
            .await
            .expect("Path invalid, should have been validated earlier");

        let raw_mesh_data = Self::combine_eligible_meshes(meta_mesh);

        let parent_dir = path.parent().expect("File must be in directory");

        let mut mesh_data = MeshData::<C>::new();
        for (vertices, indices, texture_id) in raw_mesh_data.meshes {
            let handle = client.lock().await.add_mesh(vertices, &indices);
            mesh_data.handles.push((handle, texture_id))
        }

        for texture in raw_mesh_data.textures {
            let path = resolve_path(parent_dir, PathBuf::from(&texture))
                .await
                .unwrap_or_else(|| panic!("Could not find texture {}", texture));
            mesh_data.textures.push(path_set.insert(path).await);
        }

        trace!("Loaded mesh {}", path.display());

        mesh_data
    }

    pub async fn load_mesh(&self, client: &Mutex<C>, path_set: &PathSet, path: PathBuf) -> Option<MeshData<C>> {
        let canonicalized = path.canonicalize().await.ok()?;
        let path_handle = path_set.insert(canonicalized.clone()).await;

        trace!("Checking if mesh {} is loaded", path_handle.0);
        let mesh = self
            .inner
            .get_or_insert(path_handle, async {
                self.load_mesh_impl(client, path_set, &canonicalized).await
            })
            .await;
        Some(mesh)
    }

    pub async fn remove_mesh(&self, client: &Mutex<C>, path_handle: PathHandle) -> Option<Vec<PathHandle>> {
        trace!("Checking mesh {}", path_handle.0);
        self.inner
            .remove(path_handle, async move |data| {
                trace!("Removing mesh {}", path_handle.0);
                let mut client_lock = client.lock().await;
                for (handle, _) in data.handles {
                    client_lock.remove_mesh(&handle);
                }
                data.textures
            })
            .await
    }
}
