use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectHandle(pub(crate) u64);

pub struct Object {
    pub vertex_buffer: Buffer,

    pub index_buffer: Buffer,
    pub index_count: u32,

    pub texture: u64,

    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,

    pub location: Vector3<f32>,
    pub mesh_center_offset: Vector3<f32>,
    pub camera_distance: f32,

    pub transparent: bool,
}

pub fn convert_mesh_verts_to_render_verts(
    verts: Vec<MeshVertex>,
    mut indices: Vec<u32>,
) -> (Vec<render::Vertex>, Vec<u32>) {
    // First add the extra faces due to doubling
    let mut extra_indices = Vec::new();

    for (&i1, &i2, &i3) in indices.iter().tuples() {
        let v1_double = verts[i1 as usize].double_sided;
        let v2_double = verts[i2 as usize].double_sided;
        let v3_double = verts[i3 as usize].double_sided;

        if v1_double || v2_double || v3_double {
            extra_indices.push(i3);
            extra_indices.push(i2);
            extra_indices.push(i1);
        }
    }

    // Then convert the verts to the new format
    let out_verts = verts
        .into_iter()
        .map(|v| render::Vertex {
            pos: v.position.into(),
            _color: v.color.into(),
            _normal: v.normal.into(),
            _texcoord: v.coord.into(),
        })
        .collect_vec();

    indices.extend(extra_indices.into_iter());

    (out_verts, indices)
}

pub fn find_mesh_center(mesh: &[render::Vertex]) -> Vector3<f32> {
    let first = if let Some(first) = mesh.first() {
        *first
    } else {
        return Vector3::zero();
    };
    // Bounding box time baby!
    let mut max: Vector3<f32> = first.pos.into();
    let mut min: Vector3<f32> = first.pos.into();

    for vert in mesh.iter().skip(1) {
        let pos: Vector3<f32> = vert.pos.into();
        max = max.zip(pos, |left, right| left.max(right));
        min = min.zip(pos, |left, right| left.min(right));
    }

    (max + min) / 2.0
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
        mesh_verts: Vec<MeshVertex>,
        indices: &[impl ToPrimitive],
        transparent: bool,
    ) -> ObjectHandle {
        self.add_object_texture(location, mesh_verts, indices, transparent, &texture::TextureHandle(0))
    }

    pub fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
        mesh_verts: Vec<MeshVertex>,
        indices: &[impl ToPrimitive],
        transparent: bool,
        texture::TextureHandle(tex_idx): &texture::TextureHandle,
    ) -> ObjectHandle {
        let tex: &texture::Texture = &self.textures[tex_idx];

        let indices = indices
            .iter()
            .map(|i| i.to_u32().expect("Index too large (>2^32)"))
            .collect_vec();
        let (vertices, indices) = convert_mesh_verts_to_render_verts(mesh_verts, indices);

        let vertex_buffer = self
            .device
            .create_buffer_with_data(vertices.as_bytes(), BufferUsage::VERTEX);
        let index_buffer = self
            .device
            .create_buffer_with_data(indices.as_bytes(), BufferUsage::INDEX);

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
                        range: 0..64,
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

        let mesh_center_offset = find_mesh_center(&vertices);

        let handle = self.object_handle_count;
        self.object_handle_count += 1;
        self.objects.insert(handle, Object {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            texture: 0,
            bind_group,
            uniform_buffer,
            location,
            mesh_center_offset,
            camera_distance: 0.0, // calculated later
            transparent,
        });
        ObjectHandle(handle)
    }
}
