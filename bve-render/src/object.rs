use crate::{
    camera::{FAR_PLANE_DISTANCE, NEAR_PLANE_DISTANCE},
    *,
};
use glam::{Mat4, Vec3};
use slotmap::Key;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectHandle(pub(crate) DefaultKey);

pub struct Object {
    pub mesh: DefaultKey,
    pub texture: DefaultKey,

    pub location: Vec3,
    pub camera_distance: f32,

    pub transparent: bool,
}

pub fn perspective_matrix(fovy: f32, aspect: f32) -> Mat4 {
    Mat4::perspective_lh(fovy, aspect, NEAR_PLANE_DISTANCE, FAR_PLANE_DISTANCE)
}

pub fn generate_matrix(mx_proj: &Mat4, mx_view: &Mat4, location: Vec3) -> (Mat4, Mat4, Mat4) {
    let mx_model = Mat4::from_translation(location);
    let mx_model_view = *mx_view * mx_model;
    let mx_model_view_proj = *mx_proj * mx_model_view;
    let mx_inv_trans_model_view = mx_model_view.transpose().inverse();
    (mx_model_view_proj, mx_model_view, mx_inv_trans_model_view)
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
        let mesh: &mesh::Mesh = &self.mesh[*mesh_idx];
        let tex: &texture::Texture = if tex_idx.is_null() {
            &self.textures[self.null_texture]
        } else {
            &self.textures[*tex_idx]
        };
        let transparent = mesh.transparent || tex.transparent;

        let handle = self.objects.insert(Object {
            mesh: *mesh_idx,
            texture: *tex_idx,
            location,
            camera_distance: 0.0, // calculated later
            transparent,
        });
        ObjectHandle(handle)
    }

    pub fn set_location(&mut self, object::ObjectHandle(handle): &object::ObjectHandle, location: Vec3) {
        let object: &mut object::Object = &mut self.objects[*handle];

        object.location = location;
    }

    pub fn remove_object(&mut self, ObjectHandle(obj_idx): &ObjectHandle) {
        let _object = self.objects.remove(*obj_idx).expect("Invalid object handle");
        // Object goes out of scope
    }
}
