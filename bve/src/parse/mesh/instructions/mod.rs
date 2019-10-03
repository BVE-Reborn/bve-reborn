use crate::parse::mesh::{Error, Span, BlendMode, GlowAttenuationMode};
use crate::parse::util;
use crate::{ColorU8RGB, ColorU8RGBA};
pub use generation::*;
use cgmath::{Vector2, Vector3};
use serde::Deserialize;

mod generation;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct InstructionList {
    pub instructions: Vec<Instruction>,
    pub errors: Vec<Error>,
}

impl InstructionList {
    const fn new() -> Self {
        Self {
            instructions: Vec::new(),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub span: Span,
    pub data: InstructionData,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstructionType {
    #[serde(alias = "[meshbuilder]")]
    CreateMeshBuilder,
    #[serde(alias = "vertex")]
    AddVertex,
    #[serde(alias = "face")]
    AddFace,
    #[serde(alias = "face2")]
    AddFace2,
    Cube,
    Cylinder,
    GenerateNormals, // Ignored instruction
    #[serde(alias = "[texture]")]
    Texture, // Ignored instruction
    Translate,
    TranslateAll,
    Scale,
    ScaleAll,
    Rotate,
    RotateAll,
    Sheer,
    SheerAll,
    Mirror,
    MirrorAll,
    #[serde(alias = "color")]
    SetColor,
    #[serde(alias = "emissivecolor")]
    SetEmissiveColor,
    #[serde(alias = "blendmode")]
    SetBlendMode,
    #[serde(alias = "load")]
    LoadTexture,
    #[serde(alias = "transparent")]
    SetDecalTransparentColor,
    #[serde(alias = "coordinates")]
    SetTextureCoordinates,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionData {
    CreateMeshBuilder,
    AddVertex(AddVertex),
    AddFace(AddFace),
    Cube(Cube),
    Cylinder(Cylinder),
    Translate(Translate),
    Scale(Scale),
    Rotate(Rotate),
    Sheer(Sheer),
    Mirror(Mirror),
    SetColor(SetColor),
    SetEmissiveColor(SetEmissiveColor),
    SetBlendMode(SetBlendMode),
    LoadTexture(LoadTexture),
    SetDecalTransparentColor(SetDecalTransparentColor),
    SetTextureCoordinates(SetTextureCoordinates),
}

#[bve_derive::serde_vector_proxy]
pub struct AddVertex {
    #[default("util::some_zero_f32")]
    pub location: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub normal: Vector3<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "Vec<usize>")]
pub struct AddFace {
    pub indexes: Vec<usize>,
    pub sides: Sides,
}

impl From<Vec<usize>> for AddFace {
    fn from(v: Vec<usize>) -> Self {
        Self {
            indexes: v,
            sides: Sides::Unset,
        }
    }
}

#[bve_derive::serde_vector_proxy]
pub struct Cube {
    pub half_dim: Vector3<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Cylinder {
    pub sides: u32,
    pub upper_radius: f32,
    pub lower_radius: f32,
    pub height: f32,
}

#[bve_derive::serde_vector_proxy]
pub struct Translate {
    #[default("util::some_zero_f32")]
    pub value: Vector3<f32>,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_vector_proxy]
pub struct Scale {
    #[default("util::some_one_f32")]
    pub value: Vector3<f32>,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_vector_proxy]
pub struct Rotate {
    #[default("util::some_zero_f32")]
    pub value: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub angle: f32,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_vector_proxy]
pub struct Sheer {
    #[default("util::some_zero_f32")]
    pub direction: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub sheer: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub ratio: f32,
    #[serde(skip)]
    pub application: ApplyTo,
}

// TODO: Integrate bool deserialization into macro
#[derive(Deserialize)]
struct MirrorSerdeProxy {
    #[serde(default = "util::some_zero_u8")]
    directions_x: Option<u8>,
    #[serde(default = "util::some_zero_u8")]
    directions_y: Option<u8>,
    #[serde(default = "util::some_zero_u8")]
    directions_z: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "MirrorSerdeProxy")]
pub struct Mirror {
    pub directions: Vector3<bool>,
    #[serde(skip)]
    pub application: ApplyTo,
}

impl From<MirrorSerdeProxy> for Mirror {
    fn from(o: MirrorSerdeProxy) -> Self {
        Self {
            directions: Vector3::new(o.directions_x.map_or(false, |v| v != 0), o.directions_y.map_or(false, |v| v != 0), o.directions_z.map_or(false, |v| v != 0)),
            application: ApplyTo::Unset,
        }
    }
}

#[bve_derive::serde_vector_proxy]
pub struct SetColor {
    #[default("util::some_u8_max")]
    pub color: ColorU8RGBA,
}

#[bve_derive::serde_vector_proxy]
pub struct SetEmissiveColor {
    #[default("util::some_zero_u8")]
    pub color: ColorU8RGB,
}

#[bve_derive::serde_vector_proxy]
pub struct SetBlendMode {
    #[default("SetBlendMode::default_blend_mode")]
    pub blend_mode: BlendMode,
    #[default("util::some_zero_u16")]
    pub glow_half_distance: u16,
    #[default("SetBlendMode::default_glow_attenuation_mode")]
    pub glow_attenuation_mode: GlowAttenuationMode,
}

impl SetBlendMode {
    fn default_blend_mode() -> Option<BlendMode> {
        Some(BlendMode::Normal)
    }
    fn default_glow_attenuation_mode() -> Option<GlowAttenuationMode> {
        Some(GlowAttenuationMode::DivideExponent4)
    }
}

#[bve_derive::serde_vector_proxy]
pub struct LoadTexture {
    #[default("util::some_string")]
    pub daytime: String,
    #[default("util::some_string")]
    pub nighttime: String,
}

#[bve_derive::serde_vector_proxy]
pub struct SetDecalTransparentColor {
    #[default("util::some_zero_u8")]
    pub color: ColorU8RGB,
}

#[bve_derive::serde_vector_proxy]
pub struct SetTextureCoordinates {
    pub index: usize,
    pub coords: Vector2<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sides {
    Unset,
    One,
    Two,
}

impl Default for Sides {
    fn default() -> Self {
        Self::Unset
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApplyTo {
    Unset,
    SingleMesh,
    AllMeshes,
}

impl Default for ApplyTo {
    fn default() -> Self {
        Self::Unset
    }
}
