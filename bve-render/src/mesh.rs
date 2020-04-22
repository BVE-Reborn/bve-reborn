use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub(crate) u64);

pub struct Mesh {
    pub vertex_buffer: Buffer,

    pub index_buffer: Buffer,
    pub index_count: u32,

    pub mesh_center_offset: Vector3<f32>,
    pub transparent: bool,
}

pub fn is_mesh_transparent(mesh: &[MeshVertex]) -> bool {
    mesh.iter().any(|v| v.color.w != 0 && v.color.w != 255)
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

impl Renderer {
    pub fn add_mesh(&mut self, mesh_verts: Vec<MeshVertex>, indices: &[impl ToPrimitive]) -> MeshHandle {
        let transparent = is_mesh_transparent(&mesh_verts);
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

        let mesh_center_offset = find_mesh_center(&vertices);

        let handle = self.mesh_handle_count;
        self.mesh_handle_count += 1;
        self.mesh.insert(handle, Mesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            mesh_center_offset,
            transparent,
        });

        MeshHandle(handle)
    }

    pub fn remove_mesh(&mut self, MeshHandle(mesh_idx): &MeshHandle) {
        let _mesh = self.mesh.remove(mesh_idx).expect("Invalid mesh handle");
        // Mesh goes out of scope
    }
}
