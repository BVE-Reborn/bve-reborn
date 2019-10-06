use crate::parse::mesh::instructions::*;
use crate::parse::mesh::*;
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Array, Basis3, Deg, ElementWise, InnerSpace, Rad, Rotation, Rotation3, Vector3, Zero};
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

impl Executable for Cylinder {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
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
        let face_data = ExpandedFaceData::with_defaults(Sides::One);

        let split = (n - 1).max(0) as usize;
        for i in 0..split {
            ctx.polygons.push(PolygonFace {
                face_data,
                indices: smallvec![v + (2 * i + 2), v + (2 * i + 3), v + (2 * i + 1), v + (2 * i + 0)],
            });
            ctx.polygons.push(PolygonFace {
                face_data,
                indices: smallvec![v + 0, v + 1, v + (2 * i + 1), v + (2 * i + 0)],
            });
        }
    }
}

impl Executable for Translate {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        // Handle current mesh
        for v in &mut ctx.vertices {
            v.position += self.value;
        }

        // Handle other meshes
        match self.application {
            ApplyTo::SingleMesh => {}
            ApplyTo::AllMeshes => {
                for m in &mut ctx.pso.meshes {
                    for v in &mut m.vertices {
                        v.position += self.value
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Executable for Scale {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        // Handle current mesh
        for v in &mut ctx.vertices {
            v.position = v.position.mul_element_wise(self.value);
        }

        // Handle other meshes
        match self.application {
            ApplyTo::SingleMesh => {}
            ApplyTo::AllMeshes => {
                for m in &mut ctx.pso.meshes {
                    for v in &mut m.vertices {
                        v.position = v.position.mul_element_wise(self.value);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Executable for Rotate {
    fn execute(&self, span: Span, ctx: &mut MeshBuildContext) {
        let axis = if self.axis.is_zero() {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            self.axis.normalize()
        };

        let rotation = Basis3::from_axis_angle(axis, Rad(self.angle.to_radians()));

        // Handle current mesh
        for v in &mut ctx.vertices {
            v.position = rotation.rotate_vector(v.position);
        }

        // Handle other meshes
        match self.application {
            ApplyTo::SingleMesh => {}
            ApplyTo::AllMeshes => {
                for m in &mut ctx.pso.meshes {
                    for v in &mut m.vertices {
                        v.position = rotation.rotate_vector(v.position);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
