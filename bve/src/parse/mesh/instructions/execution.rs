use crate::parse::mesh::instructions::*;
use crate::parse::mesh::*;
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Array, Basis3, ElementWise, InnerSpace, Rad, Rotation, Rotation3, Vector3, Zero};
use itertools::Itertools;
use smallvec::SmallVec;
use std::f32::consts::PI;

trait Executable {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext);
}

struct MeshBuildContext {
    pso: ParsedStaticObject,
    vertices: Vec<Vertex>,
    polygons: SmallVec<[PolygonFace; 16]>,
}

struct PolygonFace {
    indices: SmallVec<[usize; 8]>,
    face_data: ExtendedFaceData,
}

struct ExtendedFaceData {
    face_data: FaceData,
    texture_id: Option<usize>,
    color: ColorU8RGBA,
    decal_transparent_color: Option<ColorU8RGB>,
    blend_mode: BlendMode,
    glow: Glow,
    sides: Sides,
}

impl ExtendedFaceData {
    fn with_defaults(sides: Sides) -> Self {
        Self {
            face_data: FaceData::default(),
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
}

impl Instruction {
    fn execute(&self, ctx: &mut MeshBuildContext) {
        match &self.data {
            InstructionData::CreateMeshBuilder(data) => data.execute(self.span, ctx),
            InstructionData::AddVertex(data) => data.execute(self.span, ctx),
            InstructionData::AddFace(data) => data.execute(self.span, ctx),
            InstructionData::Cube(data) => data.execute(self.span, ctx),
            InstructionData::Cylinder(data) => data.execute(self.span, ctx),
            InstructionData::Translate(data) => data.execute(self.span, ctx),
            InstructionData::Scale(data) => data.execute(self.span, ctx),
            InstructionData::Rotate(data) => data.execute(self.span, ctx),
            InstructionData::Sheer(data) => data.execute(self.span, ctx),
            InstructionData::Mirror(data) => data.execute(self.span, ctx),
            InstructionData::SetColor(data) => data.execute(self.span, ctx),
            InstructionData::SetEmissiveColor(data) => data.execute(self.span, ctx),
            InstructionData::SetBlendMode(data) => data.execute(self.span, ctx),
            InstructionData::LoadTexture(data) => data.execute(self.span, ctx),
            InstructionData::SetDecalTransparentColor(data) => data.execute(self.span, ctx),
            InstructionData::SetTextureCoordinates(data) => data.execute(self.span, ctx),
        }
    }
}

impl Executable for CreateMeshBuilder {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        let key_f = |v: &PolygonFace| {
            // sort to keep faces with identical traits together
            let e = &v.face_data;
            (
                e.texture_id,
                e.decal_transparent_color.map(ColorU8RGB::sum),
                e.blend_mode,
                e.glow,
                e.color.sum(),
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
            let mut peek = group.peekable();

            let first: &PolygonFace = peek.peek().expect("Groups must not be empty");
            let first_face_data: &ExtendedFaceData = &first.face_data;

            let mesh = Mesh {
                texture: Texture {
                    texture_id: first_face_data.texture_id,
                    decal_transparent_color: first_face_data.decal_transparent_color,
                },
                color: first_face_data.color,
                blend_mode: first_face_data.blend_mode,
                glow: first_face_data.glow,
                face_data: vec![],
                vertices: vec![],
                indices: vec![],
            };
        }
    }
}

impl Executable for AddVertex {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        ctx.vertices
            .push(Vertex::from_position_normal(self.position, self.normal))
    }
}

impl Executable for AddFace {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        // Validate all indexes are in bounds
        for &idx in &self.indexes {
            if idx >= ctx.vertices.len() {
                ctx.pso.errors.push(MeshError {
                    span,
                    kind: MeshErrorKind::OutOfBounds { idx },
                });
                return;
            }
        }
        ctx.polygons.push(PolygonFace {
            face_data: ExtendedFaceData::with_defaults(self.sides),
            indices: self.indexes.clone(),
        });
    }
}

impl Executable for Cube {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        // http://openbve-project.net/documentation/HTML/object_cubecylinder.html

        use smallvec::smallvec; // Workaround for https://github.com/intellij-rust/intellij-rust/issues/4500, move to top level when fixed.

        let vertex_offset = ctx.vertices.len();

        let x = self.half_dim.x;
        let y = self.half_dim.y;
        let z = self.half_dim.z;

        ctx.vertices.reserve(8);

        ctx.vertices.push(Vertex::from_position(Vector3::new(x, y, -z)));
        ctx.vertices.push(Vertex::from_position(Vector3::new(x, -y, -z)));
        ctx.vertices.push(Vertex::from_position(Vector3::new(-x, -y, -z)));
        ctx.vertices.push(Vertex::from_position(Vector3::new(-x, y, -z)));
        ctx.vertices.push(Vertex::from_position(Vector3::new(x, y, z)));
        ctx.vertices.push(Vertex::from_position(Vector3::new(x, -y, z)));
        ctx.vertices.push(Vertex::from_position(Vector3::new(-x, -y, z)));
        ctx.vertices.push(Vertex::from_position(Vector3::new(-x, y, z)));

        ctx.polygons.reserve(6);

        let vo = vertex_offset;

        ctx.polygons.push(PolygonFace {
            face_data: ExtendedFaceData::with_defaults(Sides::One),
            indices: smallvec![vo + 0, vo + 1, vo + 2, vo + 3],
        });
        ctx.polygons.push(PolygonFace {
            face_data: ExtendedFaceData::with_defaults(Sides::One),
            indices: smallvec![vo + 0, vo + 4, vo + 5, vo + 1],
        });
        ctx.polygons.push(PolygonFace {
            face_data: ExtendedFaceData::with_defaults(Sides::One),
            indices: smallvec![vo + 0, vo + 3, vo + 7, vo + 4],
        });
        ctx.polygons.push(PolygonFace {
            face_data: ExtendedFaceData::with_defaults(Sides::One),
            indices: smallvec![vo + 6, vo + 5, vo + 4, vo + 7],
        });
        ctx.polygons.push(PolygonFace {
            face_data: ExtendedFaceData::with_defaults(Sides::One),
            indices: smallvec![vo + 6, vo + 7, vo + 3, vo + 2],
        });
        ctx.polygons.push(PolygonFace {
            face_data: ExtendedFaceData::with_defaults(Sides::One),
            indices: smallvec![vo + 6, vo + 2, vo + 1, vo + 5],
        });
    }
}

impl Executable for Cylinder {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        // http://openbve-project.net/documentation/HTML/object_cubecylinder.html

        use smallvec::smallvec; // Workaround for https://github.com/intellij-rust/intellij-rust/issues/4500, move to top level when fixed.

        let vertex_offset = ctx.vertices.len();

        // Convert args to format used in above documentation
        let n = self.sides;
        let n_f32 = n as f32;
        let r1 = self.upper_radius;
        let r2 = self.lower_radius;
        let h = self.height;

        // Vertices

        ctx.vertices.reserve(2 * (n as usize));
        for i in (0..n).map(|i| i as f32) {
            let trig_arg = (2.0 * PI * i) / n_f32;
            let cos = trig_arg.cos();
            let sin = trig_arg.sin();
            ctx.vertices
                .push(Vertex::from_position(Vector3::new(cos * r1, h / 2.0, sin * r1)));
            ctx.vertices
                .push(Vertex::from_position(Vector3::new(cos * r2, -h / 2.0, sin * r2)));
        }

        // Faces

        ctx.polygons.reserve(n as usize);

        let v = vertex_offset;

        let split = (n - 1).max(0) as usize;
        for i in 0..split {
            ctx.polygons.push(PolygonFace {
                face_data: ExtendedFaceData::with_defaults(Sides::One),
                indices: smallvec![v + (2 * i + 2), v + (2 * i + 3), v + (2 * i + 1), v + (2 * i + 0)],
            });
            ctx.polygons.push(PolygonFace {
                face_data: ExtendedFaceData::with_defaults(Sides::One),
                indices: smallvec![v + 0, v + 1, v + (2 * i + 1), v + (2 * i + 0)],
            });
        }
    }
}

/// Preform a per-vertex transform on all meshes in the MeshBuildContext depending on ApplyTo
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
            for m in &mut ctx.pso.meshes {
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

impl Executable for Sheer {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        apply_transform(self.application, ctx, |v| {
            let scale = self.ratio * (self.direction.mul_element_wise(v.position)).sum();
            v.position += self.sheer * scale;
        });
    }
}

impl Executable for Mirror {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        let factor = self.directions.map(|b| if b { -1.0f32 } else { 1.0f32 });

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
        func(&mut ctx.pso.textures, face_data)
    }
}

impl Executable for SetColor {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        edit_face_data(ctx, |_, f| f.color = self.color);
    }
}

impl Executable for SetEmissiveColor {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        edit_face_data(ctx, |_, f| f.face_data.emission_color = self.color);
    }
}

impl Executable for SetBlendMode {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        edit_face_data(ctx, |_, f| {
            f.blend_mode = self.blend_mode;
            f.glow = Glow {
                attenuation_mode: self.glow_attenuation_mode,
                half_distance: self.glow_half_distance,
            };
        });
    }
}

impl Executable for LoadTexture {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        edit_face_data(ctx, |textures, f| f.texture_id = Some(textures.add(&self.daytime)))
    }
}

impl Executable for SetDecalTransparentColor {
    fn execute(&self, _span: Span, ctx: &mut MeshBuildContext) {
        edit_face_data(ctx, |_, f| {
            f.decal_transparent_color = Some(self.color);
        })
    }
}

impl Executable for SetTextureCoordinates {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        // Validate index are in bounds
        if self.index >= ctx.vertices.len() {
            ctx.pso.errors.push(MeshError {
                span,
                kind: MeshErrorKind::OutOfBounds { idx: self.index },
            });
            return;
        }

        ctx.vertices[self.index].coord = self.coords;
    }
}
