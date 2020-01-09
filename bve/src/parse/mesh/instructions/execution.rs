use crate::parse::mesh::instructions::*;
use crate::parse::mesh::*;
use crate::Asu32;
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Array, Basis3, ElementWise, InnerSpace, Rad, Rotation, Rotation3, Vector3, Zero};
use itertools::Itertools;
use smallvec::SmallVec;
use std::f32::consts::PI;

trait Executable {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext);
}

#[derive(Debug, Default)]
struct MeshBuildContext {
    parsed: ParsedStaticObject,
    vertices: Vec<Vertex>,
    current_mesh: Mesh,
}

fn with_defaults(sides: Sides) -> Self {
    Self {
        emission_color: ColorU8RGB::from_value(0),
        texture_id: None,
        color: ColorU8RGBA::from_value(255),
        decal_transparent_color: None,
        blend_mode: BlendMode::Normal,
        glow: Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        },
        sides,
    }
}

impl Instruction {
    fn execute(&self, ctx: &mut MeshBuildContext) {
        match &self.data {
            InstructionData::CreateMeshBuilder(data) => data.execute(self.span, ctx),
            InstructionData::AddVertex(data) => data.execute(self.span, ctx),
            InstructionData::AddFace(data) => data.execute(self.span, ctx),
            InstructionData::Cube(data) => panic!("Cube instruction cannot be executed, must be postprocessed away"),
            InstructionData::Cylinder(data) => {
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
            InstructionData::SetTextureCoordinates(data) => {
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
    let key_f = |v: &PolygonFace| {
        // sort to keep faces with identical traits together
        let e = &v.face_data;
        (
            e.texture_id,
            e.decal_transparent_color.map(ColorU8RGB::as_u32),
            e.blend_mode,
            e.emission_color.as_u32(),
            e.glow,
            e.color.as_u32(),
            e.sides,
        )
    };

    ctx.polygons.sort_unstable_by_key(key_f);

    if ctx.polygons.is_empty() {
        return;
    }

    for (_key, group) in &ctx.polygons.iter().group_by(|&v| key_f(v)) {
        // Type hint
        let group: itertools::Group<'_, _, _, _> = group;
        let mut group = group.peekable();

        let first: &PolygonFace = group.peek().expect("Groups must not be empty");
        // All of these attributes are the same, so we can just grab them from the first one.
        let first_face_data: &ExtendedFaceData = &first.face_data;

        let mut mesh = Mesh {
            texture: Texture {
                texture_id: first_face_data.texture_id,
                decal_transparent_color: first_face_data.decal_transparent_color,
                emission_color: first_face_data.emission_color,
            },
            color: first_face_data.color,
            blend_mode: first_face_data.blend_mode,
            glow: first_face_data.glow,
            vertices: vec![],
            indices: vec![],
        };

        // THe entire group has all the same properties, so they can be combined into a single mesh.
        group.for_each(|face| {
            // This is a direct indices -> indices translation
            let indices = flat_shading(&ctx.vertices, &face.indices);
            let triangle_indexes = triangulate_faces(&face.indices);
            mesh.indices.extend(triangle_indexes);
        });

        mesh.vertices = shrink_vertex_list(&ctx.vertices, &mut mesh.indices);

        let (verts, indices) = flat_shading(&mesh.vertices, &mesh.indices);
        mesh.vertices = verts;
        mesh.indices = indices;

        calculate_normals(&mut mesh);

        ctx.parsed.meshes.push(mesh);
    }

    ctx.vertices.clear();
    ctx.polygons.clear();
}

impl Executable for CreateMeshBuilder {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        run_create_mesh_builder(ctx);
    }
}

impl Executable for AddVertex {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        ctx.vertices
            .push(Vertex::from_position_normal(self.position, self.normal))
    }
}

fn add_face(ctx: &mut MeshBuildContext, span: Span, sides: Sides, indices: &[usize]) {
    // Validate all indexes are in bounds
    for &idx in &indices {
        if idx >= ctx.vertices.len() {
            ctx.parsed.errors.push(MeshError {
                span,
                kind: MeshErrorKind::OutOfBounds { idx },
            });
            return;
        }
    }

    // Use my indexes to find all vertices that are only mine
    let (verts, indices) = shrink_vertex_list(&ctx.vertices, &indexes);

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
    indices
        .iter_mut()
        .for_each(|mut i| i += ctx.current_mesh.vertices.len());

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

fn edit_face_data<F>(ctx: &mut MeshBuildContext, mut func: F)
where
    F: FnMut(&mut TextureFileSet, &mut ExtendedFaceData) -> (),
{
    for f in &mut ctx.polygons {
        // CLion fails at type deduction here, help it out
        let face_data: &mut ExtendedFaceData = &mut f.face_data;
        func(&mut ctx.parsed.textures, face_data)
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

impl Executable for SetTextureCoordinates {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        if let Some(v) = ctx.vertices.get_mut(self.index) {
            v.coord = self.coords;
        } else {
            ctx.parsed.errors.push(MeshError {
                span,
                kind: MeshErrorKind::OutOfBounds { idx: self.index },
            });
        }
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
