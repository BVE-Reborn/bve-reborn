use crate::{
    camera::{FAR_PLANE_DISTANCE, NEAR_PLANE_DISTANCE},
    *,
};
use glam::{Mat4, Vec3};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectHandle(pub(crate) u64);

pub struct Object {
    pub mesh: u64,
    pub texture: u64,

    pub location: Vec3,
    pub camera_distance: f32,

    pub transparent: bool,
}

// fn perspective_dx(vertical_fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Mat4 {
//     let tan_half_fovy = (vertical_fov / 2.0).tan();
//
//     let zeros = Vec4::zero();
//     let mut result = Mat4::new(zeros, zeros, zeros, zeros);
//     result[0][0] = 1.0 / (aspect_ratio * tan_half_fovy);
//     result[1][1] = 1.0 / tan_half_fovy;
//     result[2][2] = z_far / (z_far - z_near);
//     result[2][3] = 1.0;
//     result[3][2] = -(z_far * z_near) / (z_far - z_near);
//     result
// }
//
// pub fn perspective_dx2(vertical_fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Mat4 {
//     let t = (vertical_fov / 2.0).tan();
//     let sy = 1.0 / t;
//     let sx = sy / aspect_ratio;
//     let nmf = z_near - z_far;
//
//     Mat4::new(
//         Vec4::new(sx, 0.0, 0.0, 0.0),
//         Vec4::new(0.0, sy, 0.0, 0.0),
//         Vec4::new(0.0, 0.0, z_far / nmf, 1.0),
//         Vec4::new(0.0, 0.0, z_near * z_far / nmf, 0.0),
//     )
// }

pub fn perspective_matrix(fovy: f32, aspect: f32) -> Mat4 {
    Mat4::perspective_lh(fovy, aspect, NEAR_PLANE_DISTANCE, FAR_PLANE_DISTANCE)
}

pub fn generate_matrix(mx_proj: &Mat4, mx_view: &Mat4, location: Vec3) -> (Mat4, Mat4) {
    let mx_model = Mat4::from_translation(location);
    let mx_model_view = *mx_view * mx_model;
    let mx_model_view_proj = *mx_proj * mx_model_view;
    (mx_model_view_proj, mx_model_view)
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

    pub fn set_location(&mut self, object::ObjectHandle(handle): &object::ObjectHandle, location: Vec3) {
        let object: &mut object::Object = &mut self.objects[handle];

        object.location = location;
    }

    pub fn remove_object(&mut self, ObjectHandle(obj_idx): &ObjectHandle) {
        let _object = self.objects.remove(obj_idx).expect("Invalid object handle");
        // Object goes out of scope
    }
}
