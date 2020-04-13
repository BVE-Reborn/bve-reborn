use crate::load::mesh::Vertex;
use cgmath::Vector3;
use image::RgbaImage;
use std::hash::Hash;

pub trait Client: Send + Sync + 'static {
    type ObjectHandle: Clone + Hash + Send + Sync + 'static;
    type MeshHandle: Clone + Hash + Send + Sync + 'static;
    type TextureHandle: Clone + Default + Hash + Send + Sync + 'static;

    fn add_object(&mut self, location: Vector3<f32>, mesh: &Self::MeshHandle, transparent: bool) -> Self::ObjectHandle;
    fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
        mesh: &Self::MeshHandle,
        texture: &Self::TextureHandle,
        transparent: bool,
    ) -> Self::ObjectHandle;
    fn add_mesh(&mut self, mesh_verts: Vec<Vertex>, indices: &[usize]) -> Self::MeshHandle;
    fn add_texture(&mut self, image: &RgbaImage) -> Self::TextureHandle;
}
