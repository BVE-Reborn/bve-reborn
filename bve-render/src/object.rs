use crate::*;
use std::mem::size_of;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectHandle(pub(crate) u64);

pub struct Object {
    pub mesh: u64,
    pub texture: u64,

    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,

    pub location: Vector3<f32>,
    pub camera_distance: f32,

    pub transparent: bool,
}

pub fn generate_matrix(mx_view: &Matrix4<f32>, location: Vector3<f32>, aspect_ratio: f32) -> Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(55_f32), aspect_ratio, 0.1, 1000.0);
    let mx_model = Matrix4::from_translation(location);
    OPENGL_TO_WGPU_MATRIX * mx_projection * mx_view * mx_model
}

impl Renderer {
    pub fn add_object(
        &mut self,
        location: Vector3<f32>,
        mesh_handle: &mesh::MeshHandle,
        transparent: bool,
    ) -> ObjectHandle {
        self.add_object_texture(location, mesh_handle, &texture::TextureHandle(0), transparent)
    }

    pub fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
        mesh::MeshHandle(mesh_idx): &mesh::MeshHandle,
        texture::TextureHandle(tex_idx): &texture::TextureHandle,
        transparent: bool,
    ) -> ObjectHandle {
        let tex: &texture::Texture = &self.textures[tex_idx];

        let matrix = generate_matrix(&self.camera.compute_matrix(), location, 800.0 / 600.0);
        let matrix_ref: &[f32; 16] = matrix.as_ref();
        let uniforms = render::Uniforms {
            _matrix: *matrix_ref,
            _transparent: transparent as u32,
        };
        let uniform_buffer = self
            .device
            .create_buffer_with_data(uniforms.as_bytes(), BufferUsage::UNIFORM | BufferUsage::COPY_DST);

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..(size_of::<render::Uniforms>() as u64),
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&tex.texture_view),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
            label: None,
        });

        let handle = self.object_handle_count;
        self.object_handle_count += 1;
        self.objects.insert(handle, Object {
            mesh: *mesh_idx,
            texture: *tex_idx,
            bind_group,
            uniform_buffer,
            location,
            camera_distance: 0.0, // calculated later
            transparent,
        });
        ObjectHandle(handle)
    }
}
