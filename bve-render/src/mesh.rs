use crate::*;
use glam::Vec3;
use log::trace;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub(crate) DefaultKey);

pub struct Mesh {
    pub vertex_buffer: Buffer,

    pub index_buffer: Buffer,
    pub index_count: u32,

    pub mesh_center_offset: Vec3,
    pub mesh_bounding_sphere_radius: f32,
    pub transparent: bool,
}

pub fn is_mesh_transparent(mesh: &[MeshVertex]) -> bool {
    mesh.iter().any(|v| v.color.w != 0 && v.color.w != 255)
}

pub fn convert_mesh_verts_to_render_verts(
    mut verts: Vec<MeshVertex>,
    mut indices: Vec<u32>,
) -> (Vec<render::Vertex>, Vec<u32>) {
    // First add the extra faces due to doubling
    let mut extra_indices = Vec::new();

    for (&i1, &i2, &i3) in indices.iter().tuples() {
        let mut vert1 = verts[i3 as usize];
        let mut vert2 = verts[i2 as usize];
        let mut vert3 = verts[i1 as usize];

        let v1_double = vert1.double_sided;
        let v2_double = vert2.double_sided;
        let v3_double = vert3.double_sided;

        if v1_double || v2_double || v3_double {
            vert1.normal *= -1.0;
            vert2.normal *= -1.0;
            vert3.normal *= -1.0;
            let vert_len = verts.len() as u32;
            verts.push(vert1);
            verts.push(vert2);
            verts.push(vert3);
            extra_indices.push(vert_len);
            extra_indices.push(vert_len + 1);
            extra_indices.push(vert_len + 2);
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

pub fn find_mesh_center(mesh: &[render::Vertex]) -> Vec3 {
    let first = if let Some(first) = mesh.first() {
        *first
    } else {
        return Vec3::zero();
    };
    // Bounding box time baby!
    let mut max = Vec3::from(first.pos);
    let mut min = Vec3::from(first.pos);

    for vert in mesh.iter().skip(1) {
        let pos = Vec3::from(vert.pos);
        max = max.max(pos);
        min = min.min(pos);
    }

    (max + min) / 2.0
}

pub fn find_mesh_bounding_sphere_radius(mesh_center: Vec3, mesh: &[render::Vertex]) -> f32 {
    mesh.iter().fold(0.0, |distance, vert| {
        distance.max((Vec3::from(vert.pos) - mesh_center).length())
    })
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
        let mesh_bounding_sphere_radius = find_mesh_bounding_sphere_radius(mesh_center_offset, &vertices);

        let handle = self.mesh.insert(Mesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            mesh_center_offset,
            mesh_bounding_sphere_radius,
            transparent,
        });

        trace!("Adding new mesh {:?}", handle);
        MeshHandle(handle)
    }

    pub fn remove_mesh(&mut self, MeshHandle(mesh_idx): &MeshHandle) {
        let _mesh = self.mesh.remove(*mesh_idx).expect("Invalid mesh handle");
        // Mesh goes out of scope
    }
}
