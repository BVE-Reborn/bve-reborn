use crate::parse::mesh::{BlendMode, FaceData, Glow, ParsedStaticObject, Vertex};
use crate::{ColorU8RGB, ColorU8RGBA};
use smallvec::SmallVec;

trait Executable {
    fn execute(&self, ctx: MeshBuildContext);
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
