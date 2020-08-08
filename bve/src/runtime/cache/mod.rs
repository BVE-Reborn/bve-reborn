use crate::{AsyncMutex, AsyncRwLock};
use async_std::sync::RwLock;
pub use mesh::*;
pub use path::*;
use std::{
    collections::HashMap,
    future::Future,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
pub use texture::*;

mod mesh;
mod path;
mod texture;

struct LoadedCacheEntry<D: Clone> {
    data: D,
    count: AtomicU64,
}

type CacheEntry<D> = Arc<AsyncRwLock<Option<LoadedCacheEntry<D>>>>;
struct Cache<D: Clone>(AsyncMutex<HashMap<PathHandle, CacheEntry<D>>>);

impl<D: Clone> Cache<D> {
    pub fn new() -> Self {
        Self(AsyncMutex::new(HashMap::new(), false))
    }

    pub async fn get_or_insert(&self, path_handle: PathHandle, gen_data: impl Future<Output = D>) -> D {
        // Reading/writing into the index should always be exclusive to prevent races
        let mut mutable_lock = self.0.lock().await;
        match mutable_lock.get(&path_handle) {
            Some(loaded_arc_ref) => {
                // It's already been loaded, or in the process of loading, so grab the arc to the data
                let loaded_arc = Arc::clone(loaded_arc_ref);
                // We no longer need the lock on the index
                drop(mutable_lock);

                // This read will only clear when there is data to read
                let loaded_option = loaded_arc.read().await;
                // If we get readable access to a None there's a serious programming error
                let loaded = loaded_option.as_ref().expect("Loaded mesh in cache without contents");
                // Add one to the refcount and give back the data
                loaded.count.fetch_add(1, Ordering::AcqRel);
                loaded.data.clone()
            }
            None => {
                // No one has loaded this yet. Create the data lock with None in it.
                let loaded_arc = Arc::new(RwLock::new(None));
                // Lock the RwLock for writing so no one can read it until we're done
                let mut loaded = loaded_arc.write().await;
                // Insert the Arc into the index, other threads will be able to get access to the arc,
                // but not be able to read lock it.
                mutable_lock.insert(path_handle, Arc::clone(&loaded_arc));
                // We no longer need the index
                drop(mutable_lock);

                // Load data
                let data = gen_data.await;
                // Add data to the cache
                *loaded = Some(LoadedCacheEntry {
                    data: data.clone(),
                    count: AtomicU64::new(1),
                });
                // We're done writing data, let everyone in
                drop(loaded);
                data
            }
        }
    }

    pub async fn remove<F, O>(&self, path_handle: PathHandle, remove_data: impl FnOnce(D) -> F) -> Option<O>
    where
        F: Future<Output = O>,
    {
        let mut mutable_lock = self.0.lock().await;
        if let Some(data_arc_ref) = mutable_lock.get(&path_handle) {
            let data_arc = Arc::clone(data_arc_ref);
            let data = data_arc.read().await;
            let cache_entry = data.as_ref().expect("Loaded mesh in cache without contents");
            let current_ref_count = cache_entry.count.fetch_sub(1, Ordering::AcqRel) - 1;
            drop(data);
            if current_ref_count == 0 {
                // Still have the index lock here, I am guaranteed to be the only one here
                let contents = mutable_lock.remove(&path_handle).expect("No item that we just found");
                let mut content_guard = contents.write().await;
                Some(
                    remove_data(
                        content_guard
                            .take()
                            .expect("Loaded mesh in cache without contents")
                            .data,
                    )
                    .await,
                )
            } else {
                None
            }
        } else {
            None
        }
    }
}
