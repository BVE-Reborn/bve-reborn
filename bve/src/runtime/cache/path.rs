use crate::RwLock;
use async_std::path::PathBuf;
use indexmap::IndexSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PathHandle(pub(crate) usize);

pub struct PathSet {
    inner: RwLock<IndexSet<PathBuf>>,
}

impl PathSet {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(IndexSet::new()),
        }
    }

    pub async fn insert(&self, path: PathBuf) -> PathHandle {
        debug_assert_eq!(
            Some(path.clone()),
            path.canonicalize().await.ok(),
            "Path must be canonical"
        );
        PathHandle(self.inner.write().insert_full(path).0)
    }

    pub fn get(&self, path: PathHandle) -> PathBuf {
        self.inner
            .read()
            .get_index(path.0)
            .expect("Invalid path handle")
            .clone()
    }
}
