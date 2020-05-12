//! Underlying instructions behind the parsing of a mesh.
//!
//! You may use this library if you need to specifically edit, or use the exact instructions.
//!
//! Three important functions in this library which must be run in order:
//!
//! - [`create_instructions`] takes a `&str` and parses it to instructions using a custom serde routine.
//! - [`post_process`] postprocesses away difficult to execute instructions. Must be called before execution of the
//!   instructions.
//! - [`crate::load::mesh::generate_meshes`] executes the instructions to create a mesh.
//!
//! The rest of the module is various data structures to support that.
//!
//! Makes heavy use of [`bve-derive::serde_proxy`](../../../../bve_derive/attr.serde_proxy.html) and
//! [`bve-derive::serde_vector_proxy`](../../../../bve_derive/attr.serde_vector_proxy.html)

use crate::{
    parse::{
        mesh::{BlendMode, GlowAttenuationMode, MeshError, MeshWarning},
        util, PrettyPrintResult, Span,
    },
    ColorU8RGB, ColorU8RGBA,
};
use cgmath::{Vector2, Vector3};
pub use creation::*;
pub use post_processing::*;
use serde::Deserialize;
use std::io;

mod creation;
mod post_processing;

#[derive(Debug, Clone, PartialEq)]
pub struct InstructionList {
    pub instructions: Vec<Instruction>,
    pub warnings: Vec<MeshWarning>,
    pub errors: Vec<MeshError>,
}

impl InstructionList {
    const fn new() -> Self {
        Self {
            instructions: Vec::new(),
            warnings: Vec::new(),
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

impl PrettyPrintResult for Instruction {
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        match &self.data {
            InstructionData::CreateMeshBuilder(inner) => inner.fmt(indent, out),
            InstructionData::AddVertex(inner) => inner.fmt(indent, out),
            InstructionData::AddFace(inner) => inner.fmt(indent, out),
            InstructionData::Cube(inner) => inner.fmt(indent, out),
            InstructionData::Cylinder(inner) => inner.fmt(indent, out),
            InstructionData::Translate(inner) => inner.fmt(indent, out),
            InstructionData::Scale(inner) => inner.fmt(indent, out),
            InstructionData::Rotate(inner) => inner.fmt(indent, out),
            InstructionData::Shear(inner) => inner.fmt(indent, out),
            InstructionData::Mirror(inner) => inner.fmt(indent, out),
            InstructionData::SetColor(inner) => inner.fmt(indent, out),
            InstructionData::SetEmissiveColor(inner) => inner.fmt(indent, out),
            InstructionData::SetBlendMode(inner) => inner.fmt(indent, out),
            InstructionData::LoadTexture(inner) => inner.fmt(indent, out),
            InstructionData::SetDecalTransparentColor(inner) => inner.fmt(indent, out),
            InstructionData::SetTextureCoordinates(inner) => inner.fmt(indent, out),
        }
    }
}

#[bve_derive::serde_proxy]
pub struct CreateMeshBuilder;

#[bve_derive::serde_proxy]
pub struct AddVertex {
    #[default("util::some_zero_f32")]
    pub position: Vector3<f32>,
    #[default("util::some_zero_f32")]
    pub normal: Vector3<f32>,
    /// Only relevant after postprocessing away the [`SetTextureCoordinates`] command.
    #[serde(skip)]
    pub texture_coord: Vector2<f32>,
}

#[bve_derive::serde_vector_proxy]
pub struct AddFace {
    #[primary]
    pub indexes: Vec<usize>,
    pub sides: Sides,
}

/// Cannot be executed, must be postprocessing away to [`AddVertex`] and [`AddFace`] commands
#[bve_derive::serde_proxy]
pub struct Cube {
    #[default("util::some_one_f32")]
    pub half_dim: Vector3<f32>,
}

/// Cannot be executed, must be preprocessed away to [`AddVertex`] and [`AddFace`] commands
#[bve_derive::serde_proxy]
pub struct Cylinder {
    #[default("util::some_eight_u32")]
    pub sides: u32,
    #[default("util::some_one_f32")]
    pub upper_radius: f32,
    #[default("util::some_one_f32")]
    pub lower_radius: f32,
    #[default("util::some_one_f32")]
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
    const fn default_blend_mode() -> Option<BlendMode> {
        Some(BlendMode::Normal)
    }

    const fn default_glow_attenuation_mode() -> Option<GlowAttenuationMode> {
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

/// Cannot be executed, must be preprocessed away into the corresponding [`AddVertex`] command
#[bve_derive::serde_proxy]
pub struct SetTextureCoordinates {
    #[default("util::some_zero_usize")]
    pub index: usize,
    #[default("util::some_zero_f32")]
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

impl PrettyPrintResult for Sides {
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Self::Unset => writeln!(out, "Unset"),
            Self::One => writeln!(out, "Single Sided"),
            Self::Two => writeln!(out, "Double Sided"),
        }
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

impl PrettyPrintResult for ApplyTo {
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Self::Unset => writeln!(out, "Unset"),
            Self::SingleMesh => writeln!(out, "Single Mesh"),
            Self::AllMeshes => writeln!(out, "All Meshes"),
        }
    }
}
