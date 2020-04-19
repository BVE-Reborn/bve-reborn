use crate::{
    filesystem::resolve_path,
    runtime::{
        cache::{Cache, PathHandle, PathSet},
        client::Client,
    },
};
use async_std::{
    fs::read,
    path::{Path, PathBuf},
    sync::Mutex,
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
        let tex = self
            .inner
            .get_or_insert(handle, async { self.load_texture_impl(client, &path).await })
            .await;
        Some(tex)
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
