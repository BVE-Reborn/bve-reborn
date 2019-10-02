use crate::parse::mesh::{Error, ErrorKind, FileType, Span, BlendMode, GlowAttenuationMode};
use crate::parse::util;
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Vector2, Vector3};
use csv::{ReaderBuilder, StringRecord, Trim};
use serde::Deserialize;
use std::iter::FromIterator;

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

/// Adds a comma after the first space on each line. Forces newline on last line. Lowercases string.
fn b3d_to_csv_syntax(input: &str) -> String {
    let mut p = String::with_capacity((input.len() as f32 * 1.1) as usize);
    for line in input.lines() {
        let mut lowered = line.to_lowercase();
        if let Some(idx) = lowered.find(' ') {
            lowered.replace_range(idx..idx, ",")
        }
        p.push_str(&lowered);
        p.push('\n');
    }
    if p.is_empty() {
        p.push('\n');
    }
    p
}

fn deserialize_instruction(
    inst_type: InstructionType,
    record: &StringRecord,
    span: Span,
) -> Result<Instruction, Error> {
    let data = match inst_type {
        InstructionType::CreateMeshBuilder => InstructionData::CreateMeshBuilder,
        InstructionType::AddVertex => {
            let parsed: AddVertex = record.deserialize(None)?;
            InstructionData::AddVertex(parsed)
        }
        InstructionType::AddFace => {
            let mut parsed: AddFace = record.deserialize(None)?;
            parsed.sides = Sides::One;
            InstructionData::AddFace(parsed)
        }
        InstructionType::AddFace2 => {
            let mut parsed: AddFace = record.deserialize(None)?;
            parsed.sides = Sides::Two;
            InstructionData::AddFace(parsed)
        }
        InstructionType::Cube => {
            let parsed: Cube = record.deserialize(None)?;
            InstructionData::Cube(parsed)
        }
        InstructionType::Cylinder => {
            let parsed: Cylinder = record.deserialize(None)?;
            InstructionData::Cylinder(parsed)
        }
        InstructionType::GenerateNormals => {
            return Err(Error {
                kind: ErrorKind::DeprecatedInstruction {
                    name: String::from("GenerateNormals"),
                },
                span,
            });
        }
        InstructionType::Texture => {
            return Err(Error {
                kind: ErrorKind::DeprecatedInstruction {
                    name: String::from("[texture]]"),
                },
                span,
            });
        }
        InstructionType::Translate => {
            let mut parsed: Translate = record.deserialize(None)?;
            parsed.application = ApplyTo::SingleMesh;
            InstructionData::Translate(parsed)
        }
        InstructionType::TranslateAll => {
            let mut parsed: Translate = record.deserialize(None)?;
            parsed.application = ApplyTo::AllMeshes;
            InstructionData::Translate(parsed)
        }
        InstructionType::Scale => {
            let mut parsed: Scale = record.deserialize(None)?;
            parsed.application = ApplyTo::SingleMesh;
            InstructionData::Scale(parsed)
        }
        InstructionType::ScaleAll => {
            let mut parsed: Scale = record.deserialize(None)?;
            parsed.application = ApplyTo::AllMeshes;
            InstructionData::Scale(parsed)
        }
        InstructionType::Rotate => {
            let mut parsed: Rotate = record.deserialize(None)?;
            parsed.application = ApplyTo::SingleMesh;
            InstructionData::Rotate(parsed)
        }
        InstructionType::RotateAll => {
            let mut parsed: Rotate = record.deserialize(None)?;
            parsed.application = ApplyTo::AllMeshes;
            InstructionData::Rotate(parsed)
        }
        InstructionType::Sheer => {
            let mut parsed: Sheer = record.deserialize(None)?;
            parsed.application = ApplyTo::SingleMesh;
            InstructionData::Sheer(parsed)
        }
        InstructionType::SheerAll => {
            let mut parsed: Sheer = record.deserialize(None)?;
            parsed.application = ApplyTo::AllMeshes;
            InstructionData::Sheer(parsed)
        }
        InstructionType::Mirror => {
            let mut parsed: Mirror = record.deserialize(None)?;
            parsed.application = ApplyTo::SingleMesh;
            InstructionData::Mirror(parsed)
        }
        InstructionType::MirrorAll => {
            let mut parsed: Mirror = record.deserialize(None)?;
            parsed.application = ApplyTo::AllMeshes;
            InstructionData::Mirror(parsed)
        }
        InstructionType::SetColor => {
            let parsed: SetColor = record.deserialize(None)?;
            InstructionData::SetColor(parsed)
        }
        InstructionType::SetEmissiveColor => {
            let parsed: SetEmissiveColor = record.deserialize(None)?;
            InstructionData::SetEmissiveColor(parsed)
        }
        InstructionType::SetBlendMode => {
            let parsed: SetBlendMode = record.deserialize(None)?;
            InstructionData::SetBlendMode(parsed)
        }
        InstructionType::LoadTexture => {
            let parsed: LoadTexture = record.deserialize(None)?;
            InstructionData::LoadTexture(parsed)
        }
        InstructionType::SetDecalTransparentColor => {
            let parsed: SetDecalTransparentColor = record.deserialize(None)?;
            InstructionData::SetDecalTransparentColor(parsed)
        }
        InstructionType::SetTextureCoordinates => {
            let parsed: SetTextureCoordinates = record.deserialize(None)?;
            InstructionData::SetTextureCoordinates(parsed)
        }
    };
    Ok(Instruction { data, span })
}

pub fn create_instructions(input: &str, file_type: FileType) -> InstructionList {
    // Make entire setup lowercase to make it easy to match.
    let processed = if file_type == FileType::B3D {
        b3d_to_csv_syntax(input)
    } else {
        let mut p = input.to_lowercase();
        // Ensure file ends with lowercase
        if !p.ends_with('\n') {
            p.push('\n');
        }
        p
    };

    let csv_reader = ReaderBuilder::new()
        .comment(Some(b';'))
        .has_headers(false)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(processed.as_bytes());

    let mut instructions = InstructionList::new();
    'l: for line in csv_reader.into_records() {
        match line {
            Ok(record) => {
                // Parse the instruction name
                let instruction: InstructionType = match record.get(0) {
                    Some(name) => match serde_plain::from_str(name) {
                        Ok(v) => v,
                        // Parsing fails
                        Err(_) => continue 'l,
                    },
                    // Nothing in line
                    None => continue 'l,
                };

                // Remove the already parsed instruction name
                let arguments = StringRecord::from_iter(record.iter().skip(1));
                // Get the line number
                let span = record.position().into();

                let inst = deserialize_instruction(instruction, &arguments, span);

                match inst {
                    Ok(i) => instructions.instructions.push(i),
                    Err(e) => instructions.errors.push(e),
                }
            }
            Err(_e) => {}
        }
    }

    instructions
}

#[cfg(test)]
mod tests;
