use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectHandle(pub(crate) u64);

pub struct Object {
    pub mesh: u64,
    pub texture: u64,

    pub location: Vector3<f32>,
    pub camera_distance: f32,

    pub transparent: bool,
}

pub fn generate_matrix(mx_view: &Matrix4<f32>, location: Vector3<f32>, aspect_ratio: f32) -> Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45_f32), aspect_ratio, 0.1, 1000.0);
    let mx_model = Matrix4::from_translation(location);
    OPENGL_TO_WGPU_MATRIX * mx_projection * mx_view * mx_model
}

impl Renderer {
    pub fn add_object(&mut self, location: Vector3<f32>, mesh_handle: &mesh::MeshHandle) -> ObjectHandle {
        self.add_object_texture(location, mesh_handle, &texture::TextureHandle::default())
    }

    pub fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
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
