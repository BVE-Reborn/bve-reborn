use crate::{
    filesystem::resolve_path,
    runtime::{
        cache::{PathHandle, PathSet},
        client::Client,
    },
};
use async_std::{
    fs::read,
    path::{Path, PathBuf},
    sync::Mutex,
};
use dashmap::DashMap;
use image::guess_format;
use log::trace;
use std::{
    io::Cursor,
    sync::atomic::{AtomicU64, Ordering},
};

struct LoadedTexture<C: Client> {
    handle: C::TextureHandle,
    count: AtomicU64,
}

pub struct TextureCache<C: Client> {
    inner: DashMap<PathHandle, LoadedTexture<C>>,
}

impl<C: Client> TextureCache<C> {
    pub fn new() -> Self {
        Self { inner: DashMap::new() }
    }

    async fn load_texture_impl(&self, client: &Mutex<C>, path: &Path) -> C::TextureHandle {
        trace!("Loading texture {}", path.display());

        let data = read(path).await.expect("Could not read file");

        let format = guess_format(&data).expect("Could not guess format");
        let image = image::load(Cursor::new(data), format).expect("Could not load image");
        let rgba = image.into_rgba();

        client.lock().await.add_texture(&rgba)
    }

    pub async fn load_texture_handle_path(
        &self,
        client: &Mutex<C>,
        handle: PathHandle,
        path: PathBuf,
    ) -> Option<C::TextureHandle> {
        Some(match self.inner.get(&handle) {
            Some(loaded) => {
                loaded.count.fetch_add(1, Ordering::AcqRel);
                loaded.handle.clone()
            }
            None => {
                let texture_handle = self.load_texture_impl(client, &path).await;
                self.inner.insert(handle, LoadedTexture {
                    handle: texture_handle.clone(),
                    count: AtomicU64::new(1),
                });
                texture_handle
            }
        })
    }

    pub async fn load_texture_handle(
        &self,
        client: &Mutex<C>,
        path_set: &PathSet,
        handle: PathHandle,
    ) -> Option<C::TextureHandle> {
        let path = path_set.get(handle.clone()).await;
        self.load_texture_handle_path(client, handle, path).await
    }

    #[allow(dead_code)]
    pub async fn load_texture_relative(
        &self,
        client: &Mutex<C>,
        path_set: &PathSet,
        root_dir: PathBuf,
        relative: PathBuf,
    ) -> Option<C::TextureHandle> {
        let resolved_path = resolve_path(root_dir, relative).await?;
        let handle = path_set.insert(resolved_path.clone()).await;
        self.load_texture_handle_path(client, handle, resolved_path).await
    }
}
