use crate::parse::mesh::instructions::*;
use crate::parse::mesh::*;
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Array, Basis3, ElementWise, InnerSpace, Rad, Rotation, Rotation3, Vector3, Zero};
use itertools::Itertools;

trait Executable {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext);
}

#[derive(Debug)]
struct MeshBuildContext {
    parsed: ParsedStaticObject,
    vertices: Vec<Vertex>,
    current_mesh: Mesh,
}

impl Default for MeshBuildContext {
    fn default() -> Self {
        MeshBuildContext {
            parsed: ParsedStaticObject::default(),
            vertices: Vec::default(),
            current_mesh: default_mesh(),
        }
    }
}

fn default_mesh() -> Mesh {
    Mesh {
        vertices: vec![],
        indices: vec![],
        texture: Texture {
            texture_id: None,
            emission_color: ColorU8RGB::from_value(0),
            decal_transparent_color: None,
        },
        color: ColorU8RGBA::from_value(255),
        blend_mode: BlendMode::Normal,
        glow: Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        },
    }
}

impl Instruction {
    fn execute(&self, ctx: &mut MeshBuildContext) {
        match &self.data {
            InstructionData::CreateMeshBuilder(data) => data.execute(self.span, ctx),
            InstructionData::AddVertex(data) => data.execute(self.span, ctx),
            InstructionData::AddFace(data) => data.execute(self.span, ctx),
            InstructionData::Cube(_data) => panic!("Cube instruction cannot be executed, must be postprocessed away"),
            InstructionData::Cylinder(_data) => {
                panic!("Cylinder instruction cannot be executed, must be postprocessed away")
            }
            InstructionData::Translate(data) => data.execute(self.span, ctx),
            InstructionData::Scale(data) => data.execute(self.span, ctx),
            InstructionData::Rotate(data) => data.execute(self.span, ctx),
            InstructionData::Shear(data) => data.execute(self.span, ctx),
            InstructionData::Mirror(data) => data.execute(self.span, ctx),
            InstructionData::SetColor(data) => data.execute(self.span, ctx),
            InstructionData::SetEmissiveColor(data) => data.execute(self.span, ctx),
            InstructionData::SetBlendMode(data) => data.execute(self.span, ctx),
            InstructionData::LoadTexture(data) => data.execute(self.span, ctx),
            InstructionData::SetDecalTransparentColor(data) => data.execute(self.span, ctx),
            InstructionData::SetTextureCoordinates(_data) => {
                panic!("SetTextureCoordinates instruction cannot be executed, must be postprocessed away")
            }
        }
    }
}

fn triangulate_faces(input_face: &[usize]) -> Vec<usize> {
    if input_face.len() < 3 {
        return vec![];
    }

    let face_count = input_face.len() - 2;
    let index_count = face_count * 3;

    let mut output_list = Vec::new();

    output_list.reserve(index_count);

    for i in 2..input_face.len() {
        output_list.push(input_face[0]);
        output_list.push(input_face[i - 1]);
        output_list.push(input_face[i]);
    }

    output_list
}

#[allow(clippy::identity_op)]
fn flat_shading(verts: &[Vertex], indices: &[usize]) -> (Vec<Vertex>, Vec<usize>) {
    let mut new_verts = Vec::with_capacity(indices.len());
    let mut new_indices = Vec::with_capacity(indices.len());

    for (i, (&i1, &i2, &i3)) in indices.iter().tuples().enumerate() {
        new_verts.push(verts[i1]);
        new_verts.push(verts[i2]);
        new_verts.push(verts[i3]);

        new_indices.push(3 * i + 0);
        new_indices.push(3 * i + 1);
        new_indices.push(3 * i + 2);
    }

    (new_verts, new_indices)
}

fn calculate_normals(mesh: &mut Mesh) {
    mesh.vertices
        .iter_mut()
        .for_each(|v| v.normal = Vector3::from_value(0.0));

    for (&i1, &i2, &i3) in mesh.indices.iter().tuples() {
        let a_vert: &Vertex = &mesh.vertices[i1];
        let b_vert: &Vertex = &mesh.vertices[i2];
        let c_vert: &Vertex = &mesh.vertices[i3];

        let normal = (b_vert.position - a_vert.position).cross(c_vert.position - a_vert.position);

        mesh.vertices[i1].normal += normal;
        mesh.vertices[i2].normal += normal;
        mesh.vertices[i3].normal += normal;
    }

    mesh.vertices.iter_mut().for_each(|v| v.normal = v.normal.normalize())
}

fn shrink_vertex_list(vertices: &[Vertex], indices: &[usize]) -> (Vec<Vertex>, Vec<usize>) {
    let mut translation = vec![0; vertices.len()];
    let mut vertex_used = vec![false; vertices.len()];

    let mut new_indices = indices.to_vec();

    for &mut index in new_indices.iter_mut() {
        vertex_used[index] = true;
    }

    let mut new_vertices = Vec::new();

    let mut new_index = 0_usize;
    for i in 0..(vertices.len()) {
        if !vertex_used[i] {
            continue;
        }
        new_vertices.push(vertices[i]);
        translation[i] = new_index;
        new_index += 1;
    }

    for index in new_indices.iter_mut() {
        *index = translation[*index];
    }

    (new_vertices, new_indices)
}

fn run_create_mesh_builder(ctx: &mut MeshBuildContext) {
    if ctx.current_mesh.vertices.len() != 0 && ctx.current_mesh.indices.len() != 0 {
        calculate_normals(&mut ctx.current_mesh);
        ctx.parsed
            .meshes
            .push(std::mem::replace(&mut ctx.current_mesh, default_mesh()));
    }
    ctx.vertices.clear();
}

impl Executable for CreateMeshBuilder {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        run_create_mesh_builder(ctx);
    }
}

impl Executable for AddVertex {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        ctx.vertices.push(Vertex::from_position_normal_coord(
            self.position,
            self.normal,
            self.texture_coord,
        ))
    }
}

fn add_face(ctx: &mut MeshBuildContext, span: Span, sides: Sides, indices: &[usize]) {
    // Validate all indexes are in bounds
    for &idx in indices {
        if idx >= ctx.vertices.len() {
            ctx.parsed.errors.push(MeshError {
                span,
                kind: MeshErrorKind::OutOfBounds { idx },
            });
            return;
        }
    }

    // Use my indexes to find all vertices that are only mine
    let (verts, indices) = shrink_vertex_list(&ctx.vertices, &indices);

    // Enable flat shading, make sure each vert is unique per face
    let (mut verts, indices) = flat_shading(&verts, &indices);

    // Triangulate the faces to make modern game engines not shit themselves
    let mut indices = triangulate_faces(&indices);

    // Enable the double sided flag on my vertices if I am a double sided face.
    if sides == Sides::Two {
        verts.iter_mut().for_each(|v| v.double_sided = true);
    }

    // I am going to add this to an existing list of vertices and indices, so I need to add an offset to my indices
    // so it still works
    indices.iter_mut().for_each(|i| *i += ctx.current_mesh.vertices.len());

    ctx.current_mesh.vertices.extend_from_slice(&verts);
    ctx.current_mesh.indices.extend_from_slice(&indices);
}

impl Executable for AddFace {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        add_face(ctx, span, self.sides, &self.indexes);
    }
}

/// Preform a per-vertex transform on all meshes in the [`MeshBuildContext`] depending on [`ApplyTo`]
fn apply_transform<F>(application: ApplyTo, ctx: &mut MeshBuildContext, mut func: F)
where
    F: FnMut(&mut Vertex) -> (),
{
    for v in &mut ctx.vertices {
        func(v);
    }

    // Handle other meshes
    match application {
        ApplyTo::SingleMesh => {}
        ApplyTo::AllMeshes => {
            for m in &mut ctx.parsed.meshes {
                for v in &mut m.vertices {
                    func(v);
                }
            }
        }
        _ => unreachable!(),
    }
}

impl Executable for Translate {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        apply_transform(self.application, ctx, |v| v.position += self.value);
    }
}

impl Executable for Scale {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        apply_transform(self.application, ctx, |v| {
            v.position.mul_assign_element_wise(self.value)
        });
    }
}

impl Executable for Rotate {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        let axis = if self.axis.is_zero() {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            self.axis.normalize()
        };

        let rotation = Basis3::from_axis_angle(axis, Rad(self.angle.to_radians()));

        apply_transform(self.application, ctx, |v| {
            v.position = rotation.rotate_vector(v.position);
        });
    }
}

impl Executable for Shear {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        apply_transform(self.application, ctx, |v| {
            let scale = self.ratio * (self.direction.mul_element_wise(v.position)).sum();
            v.position += self.shear * scale;
        });
    }
}

impl Executable for Mirror {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        let factor = self.directions.map(|b| if b { -1.0_f32 } else { 1.0_f32 });

        apply_transform(self.application, ctx, |v| {
            v.position.mul_assign_element_wise(factor);
        });
    }
}

impl Executable for SetColor {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        ctx.current_mesh.color = self.color;
    }
}

impl Executable for SetEmissiveColor {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        ctx.current_mesh.texture.emission_color = self.color;
    }
}

impl Executable for SetBlendMode {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        ctx.current_mesh.blend_mode = self.blend_mode;
        ctx.current_mesh.glow = Glow {
            attenuation_mode: self.glow_attenuation_mode,
            half_distance: self.glow_half_distance,
        };
    }
}

impl Executable for LoadTexture {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        ctx.current_mesh.texture.texture_id = Some(ctx.parsed.textures.add(&self.daytime));
    }
}

impl Executable for SetDecalTransparentColor {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        ctx.current_mesh.texture.decal_transparent_color = Some(self.color);
    }
}

pub fn generate_meshes(instructions: InstructionList) -> ParsedStaticObject {
    let mut mbc = MeshBuildContext::default();
    for instr in instructions.instructions {
        instr.execute(&mut mbc);
    }
    run_create_mesh_builder(&mut mbc);
    mbc.parsed.errors = instructions.errors;
    mbc.parsed
}
