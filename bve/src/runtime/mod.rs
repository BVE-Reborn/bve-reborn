use crate::load::mesh::Vertex;
use async_std::{path::Path, sync::Arc};
use cgmath::{Vector2, Vector3};

trait Client {
    type ObjectHandle;

    fn add_object(&self, location: Vector3<f32>, verts: &[Vertex], indices: &[usize]) -> Self::ObjectHandle;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
struct Location {
    pub chunk: Vector2<i32>,
    pub offset: Vector3<f32>,
}

struct Runtime<C: Client> {
    client: Arc<C>,
}

impl<C: Client> Runtime<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    pub async fn load_static_object(&self, path: &Path) {}
}
