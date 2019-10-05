use crate::parse::mesh::instructions::{Instruction, InstructionData};
use crate::parse::mesh::{BlendMode, FaceData, Glow, ParsedStaticObject, Vertex};
use crate::{ColorU8RGB, ColorU8RGBA};
use smallvec::SmallVec;

trait Executable {
    fn execute(&self, ctx: &mut MeshBuildContext);
}

struct MeshBuildContext {
    pso: ParsedStaticObject,
    vertices: Vec<Vertex>,
    untriangulated: SmallVec<[PolygonFace; 16]>,
}

struct PolygonFace {
    indices: SmallVec<[usize; 8]>,
    face_data: ExpandedFaceData,
}

struct ExpandedFaceData {
    face_data: FaceData,
    texture_id: usize,
    color: ColorU8RGBA,
    decal_transparent_color: Option<ColorU8RGB>,
    blend_mode: BlendMode,
    glow: Glow,
    double_sided: bool,
}

impl Executable for Instruction {
    fn execute(&self, ctx: &mut MeshBuildContext) {
        match &self.data {
            InstructionData::CreateMeshBuilder(data) => data.execute(),
            InstructionData::AddVertex(data) => data.execute(),
            InstructionData::AddFace(data) => data.execute(),
            InstructionData::Cube(data) => data.execute(),
            InstructionData::Cylinder(data) => data.execute(),
            InstructionData::Translate(data) => data.execute(),
            InstructionData::Scale(data) => data.execute(),
            InstructionData::Rotate(data) => data.execute(),
            InstructionData::Sheer(data) => data.execute(),
            InstructionData::Mirror(data) => data.execute(),
            InstructionData::SetColor(data) => data.execute(),
            InstructionData::SetEmissiveColor(data) => data.execute(),
            InstructionData::SetBlendMode(data) => data.execute(),
            InstructionData::LoadTexture(data) => data.execute(),
            InstructionData::SetDecalTransparentColor(data) => data.execute(),
            InstructionData::SetTextureCoordinates(data) => data.execute(),
        }
    }
}
