use crate::{
    filesystem::resolve_path,
    runtime::{
        cache::{Cache, PathHandle, PathSet},
        client::Client,
    },
    AsyncMutex,
};
use async_std::{
    fs::read,
    path::{Path, PathBuf},
};
use image::guess_format;
use log::trace;
use std::io::Cursor;

pub struct TextureCache<C: Client> {
    inner: Cache<C::TextureHandle>,
}

impl<C: Client> TextureCache<C> {
    pub fn new() -> Self {
        Self { inner: Cache::new() }
    }

    async fn load_texture_impl(&self, client: &AsyncMutex<C>, path: &Path) -> C::TextureHandle {
        trace!("Loading texture {}", path.display());

        let data = read(path).await.expect("Could not read file");

        let format = guess_format(&data).expect("Could not guess format");
        let image = image::load(Cursor::new(data), format).expect("Could not load image");
        let rgba = image.into_rgba();

        client.lock().await.add_texture(&rgba)
    }

    pub async fn load_texture_handle_path(
        &self,
        client: &AsyncMutex<C>,
        handle: PathHandle,
        path: PathBuf,
    ) -> Option<C::TextureHandle> {
        let tex = self
            .inner
            .get_or_insert(handle, async { self.load_texture_impl(client, &path).await })
            .await;
        Some(tex)
    }

    pub async fn load_texture_handle(
        &self,
        client: &AsyncMutex<C>,
        path_set: &PathSet,
        handle: PathHandle,
    ) -> Option<C::TextureHandle> {
        let path = path_set.get(handle);
        self.load_texture_handle_path(client, handle, path).await
    }

    #[allow(dead_code)]
    pub async fn load_texture_relative(
        &self,
        client: &AsyncMutex<C>,
        path_set: &PathSet,
        root_dir: PathBuf,
        relative: PathBuf,
    ) -> Option<C::TextureHandle> {
        let resolved_path = resolve_path(root_dir, relative).await?;
        let handle = path_set.insert(resolved_path.clone()).await;
        self.load_texture_handle_path(client, handle, resolved_path).await
    }

    pub async fn remove_texture(&self, client: &AsyncMutex<C>, path_handle: PathHandle) -> Option<()> {
        self.inner
            .remove(path_handle, async move |handle| {
                let mut client_lock = client.lock().await;
                client_lock.remove_texture(&handle);
            })
            .await
    }
}
