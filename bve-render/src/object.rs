use crate::*;
use nalgebra_glm::{perspective_lh_zo, translation, Mat4, Vec3};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectHandle(pub(crate) u64);

pub struct Object {
    pub mesh: u64,
    pub texture: u64,

    pub location: Vec3,
    pub camera_distance: f32,

    pub transparent: bool,
}

pub fn perspective_matrix(fovy: f32, aspect: f32) -> Mat4 {
    // let range = (fovy / 2.0).tan() * z_near;
    //
    // let left = -range * aspect;
    // let right = range * aspect;
    // let bottom = -range;
    // let top = range;
    //
    // let mut result: Mat4 = zero();
    //
    // result.m11 = (2.0 * z_near) / (right - left);
    // result.m22 = (2.0 * z_near) / (top - bottom);
    // result.m33 = 1.0;
    // result.m34 = 1.0;
    // result.m43 = -2.0 * z_near;

    // result
    perspective_lh_zo(aspect, fovy, 0.1, 10000.0)
}

pub fn generate_matrix(mx_proj: &Mat4, mx_view: &Mat4, location: Vec3) -> Mat4 {
    let mx_model = translation(&location);
    mx_proj * mx_view * mx_model
}

impl Renderer {
    pub fn add_object(&mut self, location: Vec3, mesh_handle: &mesh::MeshHandle) -> ObjectHandle {
        self.add_object_texture(location, mesh_handle, &texture::TextureHandle::default())
    }

    pub fn add_object_texture(
        &mut self,
        location: Vec3,
        mesh::MeshHandle(mesh_idx): &mesh::MeshHandle,
        texture::TextureHandle(tex_idx): &texture::TextureHandle,
    ) -> ObjectHandle {
        let mesh: &mesh::Mesh = &self.mesh[mesh_idx];
        let tex: &texture::Texture = &self.textures[tex_idx];
        let transparent = mesh.transparent || tex.transparent;

        let handle = self.object_handle_count;
        self.object_handle_count += 1;
        self.objects.insert(handle, Object {
            mesh: *mesh_idx,
            texture: *tex_idx,
            location,
            camera_distance: 0.0, // calculated later
            transparent,
        });
        ObjectHandle(handle)
    }

    pub fn remove_object(&mut self, ObjectHandle(obj_idx): &ObjectHandle) {
        let _object = self.objects.remove(obj_idx).expect("Invalid object handle");
        // Object goes out of scope
    }
}
