use crate::{
    load::mesh::*,
    panic_log,
    parse::{
        mesh::{instructions::*, *},
        Span,
    },
};
use glam::{Mat3, Vec3};
use itertools::Itertools;
use log::trace;

trait Executable {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext);
}

#[derive(Debug)]
struct MeshBuildContext {
    parsed: LoadedStaticMesh,
    vertices: Vec<Vertex>,
    current_mesh: Mesh,
}

impl Default for MeshBuildContext {
    fn default() -> Self {
        Self {
            parsed: LoadedStaticMesh::default(),
            vertices: Vec::default(),
            current_mesh: default_mesh(),
        }
    }
}

impl Instruction {
    fn execute(&self, ctx: &mut MeshBuildContext) {
        match &self.data {
            InstructionData::CreateMeshBuilder(data) => data.execute(self.span, ctx),
            InstructionData::AddVertex(data) => data.execute(self.span, ctx),
            InstructionData::AddFace(data) => data.execute(self.span, ctx),
            InstructionData::Cube(_data) => {
                panic_log!("Cube instruction cannot be executed, must be postprocessed away");
            }
            InstructionData::Cylinder(_data) => {
                panic_log!("Cylinder instruction cannot be executed, must be postprocessed away");
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
                panic_log!("SetTextureCoordinates instruction cannot be executed, must be postprocessed away");
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

fn calculate_normals(mesh: &mut Mesh) {
    mesh.vertices.iter_mut().for_each(|v| v.normal = Vec3::zero());

    for (&i1, &i2, &i3) in mesh.indices.iter().tuples() {
        let a_vert: &Vertex = &mesh.vertices[i1];
        let b_vert: &Vertex = &mesh.vertices[i2];
        let c_vert: &Vertex = &mesh.vertices[i3];

        let normal = (b_vert.position - a_vert.position).cross(c_vert.position - a_vert.position);

        mesh.vertices[i1].normal += normal;
        mesh.vertices[i2].normal += normal;
        mesh.vertices[i3].normal += normal;
    }

    mesh.vertices.iter_mut().for_each(|v| v.normal = v.normal.normalize());
}

fn shrink_vertex_list(vertices: &[Vertex], indices: &[usize]) -> (Vec<Vertex>, Vec<usize>) {
    let mut translation = vec![0; vertices.len()];
    let mut vertex_used = vec![false; vertices.len()];

    let mut new_indices = indices.to_vec();

    for &mut index in &mut new_indices {
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

    for index in &mut new_indices {
        *index = translation[*index];
    }

    (new_vertices, new_indices)
}

fn run_create_mesh_builder(ctx: &mut MeshBuildContext) {
    if !ctx.current_mesh.vertices.is_empty() && !ctx.current_mesh.indices.is_empty() {
        calculate_normals(&mut ctx.current_mesh);
        // Distribute the mesh color to the vertices. It is redundant, but when
        // these meshes get combined, this is the easiest and fastest way for shaders
        // to access the data
        for v in &mut ctx.current_mesh.vertices {
            v.color = ctx.current_mesh.color;
        }
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
                location: span,
                kind: MeshErrorKind::OutOfBounds { idx },
            });
            return;
        }
    }

    // Use my indexes to find all vertices that are only mine
    let (mut verts, indices) = shrink_vertex_list(&ctx.vertices, indices);

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
    for v in &mut ctx.current_mesh.vertices {
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
        apply_transform(self.application, ctx, |v| v.position = self.value * v.position);
    }
}

impl Executable for Rotate {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        let axis = if self.axis == Vec3::zero() {
            Vec3::unit_x()
        } else {
            self.axis.normalize()
        };

        let rotation = Mat3::from_axis_angle(axis, self.angle.to_radians());

        apply_transform(self.application, ctx, |v| {
            v.position = rotation * v.position;
        });
    }
}

impl Executable for Shear {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        apply_transform(self.application, ctx, |v| {
            let scale = self.ratio * self.direction.dot(v.position);
            v.position += self.shear * scale;
        });
    }
}

impl Executable for Mirror {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        let factor = self.directions.map_f32(|b| if b { -1.0_f32 } else { 1.0_f32 });

        apply_transform(self.application, ctx, |v| {
            v.position *= factor;
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

/// Actually execute the instructions provided.
///
/// Errors are taken from [`InstructionList::errors`] and any new ones encountered are appended and put in
/// [`ParsedStaticObject::errors`]. These errors are all non-fatal, so [`Result`] can't be used.
///
/// # Panic
///
/// Must be postprocessed.
#[must_use]
pub fn generate_meshes(instructions: InstructionList) -> LoadedStaticMesh {
    trace!("Generating mesh");
    let mut mbc = MeshBuildContext::default();
    for instr in instructions.instructions {
        instr.execute(&mut mbc);
    }
    run_create_mesh_builder(&mut mbc);
    mbc.parsed.warnings = instructions.warnings;
    mbc.parsed.errors = instructions.errors;
    mbc.parsed
}

#[cfg(test)]
mod test {
    use crate::{
        load::mesh::{execution::generate_meshes, BlendMode, Glow, GlowAttenuationMode},
        parse::{mesh::instructions::*, Span},
        ColorU8RGB, ColorU8RGBA,
    };
    use glam::{Vec2, Vec3};
    use obj::{Obj, SimplePolygon};
    use std::{
        io::{BufReader, Cursor},
        iter::FromIterator,
    };

    pub const CUBE_SOURCE: &str = include_str!("cube.obj");

    fn generate_instructions_from_obj(input: &'static str) -> InstructionList {
        let input = input.as_bytes().to_vec();
        let mut buf = BufReader::new(Cursor::new(input));

        let obj: Obj<'_, SimplePolygon> = Obj::load_buf(&mut buf).expect("Unable to parse obj");
        let mut result = vec![Instruction {
            span: Span::none(),
            data: InstructionData::CreateMeshBuilder(CreateMeshBuilder),
        }];

        // For every face, we separately create the vertices needed
        let mut index_count = 0;
        for face in &obj.objects[0].groups[0].polys {
            for (offset, vert) in face.iter().enumerate() {
                let position = obj.position[vert.0];
                let position = Vec3::from(position);
                let normal = obj.normal[vert.2.expect("OBJ must have normals")];
                let normal = Vec3::from(normal);
                result.push(Instruction {
                    span: Span::none(),
                    data: InstructionData::AddVertex(AddVertex {
                        position,
                        normal,
                        texture_coord: Vec2::zero(),
                    }),
                });
                let texture_coord = obj.texture[vert.1.expect("OBJ must have texture coords")];
                let texture_coord = Vec2::from(texture_coord);
                result.push(Instruction {
                    span: Span::none(),
                    data: InstructionData::SetTextureCoordinates(SetTextureCoordinates {
                        coords: texture_coord,
                        index: index_count + offset,
                    }),
                });
            }
            let face_vertices = face.len();
            result.push(Instruction {
                span: Span::none(),
                data: InstructionData::AddFace(AddFace {
                    indexes: Vec::from_iter(index_count..(index_count + face_vertices)),
                    sides: Sides::One,
                }),
            });
            index_count += face_vertices;
        }

        InstructionList {
            instructions: result,
            warnings: Vec::default(),
            errors: Vec::default(),
        }
    }

    macro_rules! generate_instruction_list {
    ($($num:literal: $id:ident { $($tokens:tt)* }),*) => {
        InstructionList {
            instructions: vec![$(Instruction{
                data: InstructionData::$id($id{
                    $($tokens)*
                }),
                span: Span{
                    line: Some($num)
                }
            }),+],
            warnings: vec![],
            errors: vec![],
        }
    };
}

    #[bve_derive::bve_test]
    #[test]
    fn single_mesh() {
        let v = generate_instruction_list!(
            0: CreateMeshBuilder {},
            1: AddVertex {
                position: Vec3::zero(),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            2: AddVertex {
                position: Vec3::new(-0.866_025, 0.0, 0.5),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            3: AddVertex {
                position: Vec3::new(0.866_025, 0.0, 0.5),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            4: AddFace {
                indexes: vec![0, 1, 2],
                sides: Sides::One,
            },
            5: LoadTexture {
                daytime: String::from("day_tex"),
                nighttime: String::from("night_tex"),
            }
        );

        let result = generate_meshes(post_process(v));
        assert_eq!(result.meshes.len(), 1);
        let mesh = &result.meshes[0];
        assert_eq!(mesh.vertices[0].position, Vec3::zero());
        assert_eq!(mesh.vertices[1].position, Vec3::new(-0.866_025, 0.0, 0.5));
        assert_eq!(mesh.vertices[2].position, Vec3::new(0.866_025, 0.0, 0.5));
        for v in &mesh.vertices {
            assert_eq!(v.normal, Vec3::new(0.0, 1.0, 0.0));
        }
        for &v in &mesh.vertices {
            assert_eq!(v.coord, Vec2::zero());
        }
        assert_eq!(mesh.indices, vec![0, 1, 2]);
        assert_eq!(mesh.blend_mode, BlendMode::Normal);
        assert_eq!(mesh.color, ColorU8RGBA::splat(255));
        assert_eq!(mesh.glow, Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        });
        assert_eq!(result.textures.len(), 1);
        assert_eq!(result.textures.lookup(0), Some("day_tex"));
        assert_eq!(result.errors.len(), 0);
    }

    /// First mesh uses all possible attributes besides texture coords,
    /// helps test to make sure state is properly reset
    #[bve_derive::bve_test]
    #[test]
    fn double_mesh() {
        let v = generate_instruction_list!(
            0: CreateMeshBuilder {},
            1: AddVertex {
                position: Vec3::zero(),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            2: AddVertex {
                position: Vec3::new(-0.866_025, 0.0, 0.5),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            3: AddVertex {
                position: Vec3::new(0.866_025, 0.0, 0.5),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            4: AddFace {
                indexes: vec![0, 1, 2],
                sides: Sides::One,
            },
            5: LoadTexture {
                daytime: String::from("day_tex"),
                nighttime: String::from("night_tex"),
            },
            6: SetBlendMode {
                blend_mode: BlendMode::Additive,
                glow_half_distance: 12_u16,
                glow_attenuation_mode: GlowAttenuationMode::DivideExponent2,
            },
            7: SetEmissiveColor {
                color: ColorU8RGB::new(11, 12, 13)
            },
            8: SetColor {
                color: ColorU8RGBA::new(21, 22, 23, 24)
            },
            9: SetDecalTransparentColor {
                color: ColorU8RGB::new(31, 32, 33)
            },
            10: CreateMeshBuilder {},
            11: AddVertex {
                position: Vec3::zero(),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            12: AddVertex {
                position: Vec3::new(-0.866_025, 0.0, 0.5),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            13: AddVertex {
                position: Vec3::new(0.866_025, 0.0, 0.5),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            14: AddFace {
                indexes: vec![0, 1, 2],
                sides: Sides::One,
            },
            15: LoadTexture {
                daytime: String::from("other_day_tex"),
                nighttime: String::from("night_tex"),
            }
        );

        let result = generate_meshes(post_process(v));
        assert_eq!(result.meshes.len(), 2);

        // First Mesh
        let mesh = &result.meshes[0];
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.vertices[0].position, Vec3::zero());
        assert_eq!(mesh.vertices[1].position, Vec3::new(-0.866_025, 0.0, 0.5));
        assert_eq!(mesh.vertices[2].position, Vec3::new(0.866_025, 0.0, 0.5));
        for v in &mesh.vertices {
            assert_eq!(v.normal, Vec3::unit_y());
        }
        for &v in &mesh.vertices {
            assert_eq!(v.coord, Vec2::zero());
        }
        assert_eq!(mesh.indices, vec![0, 1, 2]);
        assert_eq!(mesh.blend_mode, BlendMode::Additive);
        assert_eq!(mesh.color, ColorU8RGBA::new(21, 22, 23, 24));
        assert_eq!(mesh.texture.emission_color, ColorU8RGB::new(11, 12, 13));
        assert_eq!(mesh.texture.decal_transparent_color, Some(ColorU8RGB::new(31, 32, 33)));
        assert_eq!(mesh.glow, Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent2,
            half_distance: 12,
        });

        // Second Mesh
        let mesh = &result.meshes[1];
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.vertices[0].position, Vec3::zero());
        assert_eq!(mesh.vertices[1].position, Vec3::new(-0.866_025, 0.0, 0.5));
        assert_eq!(mesh.vertices[2].position, Vec3::new(0.866_025, 0.0, 0.5));
        for v in &mesh.vertices {
            assert_eq!(v.normal, Vec3::unit_y());
        }
        for &v in &mesh.vertices {
            assert_eq!(v.coord, Vec2::zero());
        }
        assert_eq!(mesh.indices, vec![0, 1, 2]);
        assert_eq!(mesh.blend_mode, BlendMode::Normal);
        assert_eq!(mesh.color, ColorU8RGBA::splat(255));
        assert_eq!(mesh.glow, Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        });
        assert_eq!(result.textures.len(), 2);
        assert_eq!(result.textures.lookup(0), Some("day_tex"));
        assert_eq!(result.textures.lookup(1), Some("other_day_tex"));
        assert_eq!(result.errors.len(), 0);
    }

    #[bve_derive::bve_test]
    #[test]
    fn texture_coords() {
        let v = generate_instruction_list!(
            0: CreateMeshBuilder {},
            1: AddVertex {
                position: Vec3::zero(),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            2: AddVertex {
                position: Vec3::new(-0.866_025, 0.0, 0.5),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            3: AddVertex {
                position: Vec3::new(0.866_025, 0.0, 0.5),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            },
            4: AddFace {
                indexes: vec![0, 1, 2],
                sides: Sides::One,
            },
            5: LoadTexture {
                daytime: String::from("day_tex"),
                nighttime: String::from("night_tex"),
            },
            6: SetTextureCoordinates {
                index: 0,
                coords: Vec2::splat(1.0),
            },
            7: SetTextureCoordinates {
                index: 1,
                coords: Vec2::splat(2.0),
            },
            8: SetTextureCoordinates {
                index: 2,
                coords: Vec2::splat(3.0),
            }
        );

        let result = generate_meshes(post_process(v));
        assert_eq!(result.meshes.len(), 1);
        let mesh = &result.meshes[0];
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.vertices[0].position, Vec3::zero());
        assert_eq!(mesh.vertices[1].position, Vec3::new(-0.866_025, 0.0, 0.5));
        assert_eq!(mesh.vertices[2].position, Vec3::new(0.866_025, 0.0, 0.5));
        for v in &mesh.vertices {
            assert_eq!(v.normal, Vec3::unit_y());
        }
        for (i, &v) in mesh.vertices.iter().enumerate() {
            assert_eq!(v.coord, Vec2::splat((i + 1) as f32));
        }
        assert_eq!(mesh.indices, vec![0, 1, 2]);
        assert_eq!(mesh.blend_mode, BlendMode::Normal);
        assert_eq!(mesh.color, ColorU8RGBA::splat(255));
        assert_eq!(mesh.glow, Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        });
        assert_eq!(result.textures.len(), 1);
        assert_eq!(result.textures.lookup(0), Some("day_tex"));
        assert_eq!(result.errors.len(), 0);
    }

    #[bve_derive::bve_test]
    #[test]
    fn cube() {
        let v = generate_instructions_from_obj(CUBE_SOURCE);

        let result = generate_meshes(post_process(v));
        assert_eq!(result.meshes.len(), 1);
        let mesh = &result.meshes[0];
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        assert_eq!(result.textures.len(), 0);
        assert_eq!(result.errors.len(), 0);
    }
}
