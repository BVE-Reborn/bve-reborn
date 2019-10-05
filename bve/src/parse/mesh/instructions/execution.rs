use crate::parse::mesh::instructions::*;
use crate::parse::mesh::*;
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Array, Vector3};
use smallvec::{smallvec, SmallVec};

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
    face_data: ExpandedFaceData,
}

struct ExpandedFaceData {
    face_data: FaceData,
    texture_id: Option<usize>,
    color: ColorU8RGBA,
    decal_transparent_color: Option<ColorU8RGB>,
    blend_mode: BlendMode,
    glow: Glow,
    sides: Sides,
}

impl ExpandedFaceData {
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
        unimplemented!()
    }
}

impl Executable for AddVertex {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        ctx.vertices
            .push(Vertex::from_position_normal(self.position, self.normal))
    }
}

impl Executable for AddFace {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        // Validate all indexes inbounds
        for idx in self.indexes {
            if idx >= ctx.vertices.len() {
                ctx.pso.errors.push(MeshError {
                    span,
                    kind: MeshErrorKind::OutOfBounds { idx },
                });
                return;
            }
        }
        ctx.polygons.push(PolygonFace {
            face_data: ExpandedFaceData::with_defaults(self.sides),
            indices: self.indexes.clone(),
        });
    }
}

impl Executable for Cube {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
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

        let face_data = ExpandedFaceData::with_defaults(Sides::One);
        let vo = vertex_offset;

        ctx.polygons.push(PolygonFace {
            face_data,
            indices: smallvec![vo + 0, vo + 1, vo + 2, vo + 3],
        });
        ctx.polygons.push(PolygonFace {
            face_data,
            indices: smallvec![vo + 0, vo + 4, vo + 5, vo + 1],
        });
        ctx.polygons.push(PolygonFace {
            face_data,
            indices: smallvec![vo + 0, vo + 3, vo + 7, vo + 4],
        });
        ctx.polygons.push(PolygonFace {
            face_data,
            indices: smallvec![vo + 6, vo + 5, vo + 4, vo + 7],
        });
        ctx.polygons.push(PolygonFace {
            face_data,
            indices: smallvec![vo + 6, vo + 7, vo + 3, vo + 2],
        });
        ctx.polygons.push(PolygonFace {
            face_data,
            indices: smallvec![vo + 6, vo + 2, vo + 1, vo + 5],
        });
    }
}
