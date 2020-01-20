use crate::parse::mesh::{BlendMode, GlowAttenuationMode, MeshError};
use crate::parse::{util, Span};
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Vector2, Vector3};
pub use creation::*;
pub use execution::*;
pub use post_processing::*;
use serde::Deserialize;

mod creation;
mod execution;
mod post_processing;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct InstructionList {
    pub instructions: Vec<Instruction>,
    pub errors: Vec<MeshError>,
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

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
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
    Shear,
    ShearAll,
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
    CreateMeshBuilder(CreateMeshBuilder),
    AddVertex(AddVertex),
    AddFace(AddFace),
    Cube(Cube),
    Cylinder(Cylinder),
    Translate(Translate),
    Scale(Scale),
    Rotate(Rotate),
    Shear(Shear),
    Mirror(Mirror),
    SetColor(SetColor),
    SetEmissiveColor(SetEmissiveColor),
    SetBlendMode(SetBlendMode),
    LoadTexture(LoadTexture),
    SetDecalTransparentColor(SetDecalTransparentColor),
    SetTextureCoordinates(SetTextureCoordinates),
}

#[bve_derive::serde_proxy]
pub struct CreateMeshBuilder;

#[bve_derive::serde_proxy]
pub struct AddVertex {
    #[default("util::some_zero_f32")]
    pub position: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub normal: Vector3<f32>,
    #[serde(skip)]
    pub texture_coord: Vector2<f32>,
}

#[bve_derive::serde_vector_proxy]
pub struct AddFace {
    #[primary]
    pub indexes: Vec<usize>,
    pub sides: Sides,
}

#[bve_derive::serde_proxy]
pub struct Cube {
    pub half_dim: Vector3<f32>,
}

#[bve_derive::serde_proxy]
pub struct Cylinder {
    pub sides: u32,
    pub upper_radius: f32,
    pub lower_radius: f32,
    pub height: f32,
}

#[bve_derive::serde_proxy]
pub struct Translate {
    #[default("util::some_zero_f32")]
    pub value: Vector3<f32>,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_proxy]
pub struct Scale {
    #[default("util::some_one_f32")]
    pub value: Vector3<f32>,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_proxy]
pub struct Rotate {
    #[default("util::some_zero_f32")]
    pub axis: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub angle: f32,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_proxy]
pub struct Shear {
    #[default("util::some_zero_f32")]
    pub direction: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub shear: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub ratio: f32,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_proxy]
pub struct Mirror {
    #[default("util::some_false")]
    pub directions: Vector3<bool>,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_proxy]
pub struct SetColor {
    #[default("util::some_u8_max")]
    pub color: ColorU8RGBA,
}

#[bve_derive::serde_proxy]
pub struct SetEmissiveColor {
    #[default("util::some_zero_u8")]
    pub color: ColorU8RGB,
}

#[bve_derive::serde_proxy]
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

#[bve_derive::serde_proxy]
pub struct LoadTexture {
    #[default("util::some_string")]
    pub daytime: String,
    #[default("util::some_string")]
    pub nighttime: String,
}

#[bve_derive::serde_proxy]
pub struct SetDecalTransparentColor {
    #[default("util::some_zero_u8")]
    pub color: ColorU8RGB,
}

#[bve_derive::serde_proxy]
pub struct SetTextureCoordinates {
    pub index: usize,
    pub coords: Vector2<f32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sides {
    Unset,
    One,
    Two,
}

impl Default for Sides {
    #[must_use]
    fn default() -> Self {
        Self::Unset
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApplyTo {
    Unset,
    SingleMesh,
    AllMeshes,
}

impl Default for ApplyTo {
    #[must_use]
    fn default() -> Self {
        Self::Unset
    }
}
