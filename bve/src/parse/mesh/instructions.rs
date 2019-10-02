use crate::parse::mesh::{Error, ErrorKind, FileType, Span};
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
    pub location: Vector3<f32>,
    pub normal: Vector3<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "Vec<usize>")]
pub struct AddFace {
    #[serde(flatten)]
    pub indexes: Vec<usize>,
    #[serde(skip)]
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
    pub value: Vector3<f32>,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_vector_proxy]
pub struct Scale {
    pub value: Vector3<f32>,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_vector_proxy]
pub struct Rotate {
    pub value: Vector3<f32>,
    pub angle: f32,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_vector_proxy]
pub struct Sheer {
    pub direction: Vector3<f32>,
    pub sheer: Vector3<f32>,
    pub ratio: f32,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[derive(Deserialize)]
struct MirrorSerdeProxy {
    directions_x: u8,
    directions_y: u8,
    directions_z: u8,
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
            directions: Vector3::new(o.directions_x != 0, o.directions_y != 0, o.directions_z != 0),
            application: ApplyTo::Unset,
        }
    }
}

#[bve_derive::serde_vector_proxy]
pub struct SetColor {
    pub color: ColorU8RGBA,
}

#[bve_derive::serde_vector_proxy]
pub struct SetEmissiveColor {
    pub color: ColorU8RGB,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SetBlendMode {
    pub blend_mode: BlendMode,
    pub glow_half_distance: u16,
    pub glow_attenutation_mode: GlowAttenuationMode,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LoadTexture {
    pub daytime: String,
    pub nighttime: String,
}

#[bve_derive::serde_vector_proxy]
pub struct SetDecalTransparentColor {
    pub color: ColorU8RGB,
}

#[bve_derive::serde_vector_proxy]
pub struct SetTextureCoordinates {
    pub index: usize,
    pub coords: Vector2<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    Normal,
    Additive,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlowAttenuationMode {
    DivideExponent2,
    DivideExponent4,
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

pub fn create(input: &str, file_type: FileType) -> InstructionList {
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
mod test {
    mod b3d_to_csv_syntax {
        use crate::parse::mesh::instructions::b3d_to_csv_syntax;

        #[test]
        fn comma_add() {
            assert_eq!(b3d_to_csv_syntax("myinstruction arg1"), "myinstruction, arg1\n");
        }
        #[test]
        fn multiline_comma_add() {
            assert_eq!(
                b3d_to_csv_syntax("myinstruction arg1\nmyother arg2"),
                "myinstruction, arg1\nmyother, arg2\n"
            );
        }

        #[test]
        fn spaceless() {
            assert_eq!(b3d_to_csv_syntax("myinstruction"), "myinstruction\n");
            assert_eq!(b3d_to_csv_syntax(""), "\n");
        }

        #[test]
        fn multiline_spaceless() {
            assert_eq!(b3d_to_csv_syntax("myinstruction\n\nfk2"), "myinstruction\n\nfk2\n");
            assert_eq!(b3d_to_csv_syntax("\n\n"), "\n\n");
        }
    }

    mod create_instructions {
        use crate::parse::mesh::instructions::*;
        use crate::parse::mesh::{FileType, Span};
        use crate::{ColorU8RGB, ColorU8RGBA};
        use cgmath::{Vector2, Vector3};

        macro_rules! no_instruction_assert {
            ( $inputB3D:literal, $inputCSV:literal, $args:literal ) => {
                let result_a = create(concat!($inputB3D, " ", $args).into(), FileType::B3D);
                if result_a.errors.is_empty() {
                    panic!("Missing Errors: {:#?}", result_a)
                }
                assert_eq!(result_a.instructions.len(), 0);
                let result_b = create(concat!($inputCSV, ",", $args).into(), FileType::CSV);
                if result_b.errors.is_empty() {
                    panic!("Missing Errors: {:#?}", result_b)
                }
                assert_eq!(result_b.instructions.len(), 0);
            };
        }

        macro_rules! instruction_assert {
            ( $inputB3D:literal, $inputCSV:literal, $args:literal, $data:expr ) => {
                let result_a = create(concat!($inputB3D, " ", $args).into(), FileType::B3D);
                if !result_a.errors.is_empty() {
                    panic!("ERRORS!! {:#?}", result_a)
                }
                assert_eq!(
                    *result_a.instructions.get(0).unwrap(),
                    Instruction {
                        data: $data,
                        span: Span { line: Some(1) }
                    }
                );
                let result_b = create(concat!($inputCSV, ",", $args).into(), FileType::CSV);
                if !result_b.errors.is_empty() {
                    panic!("ERRORS!! {:#?}", result_b)
                }
                assert_eq!(
                    *result_b.instructions.get(0).unwrap(),
                    Instruction {
                        data: $data,
                        span: Span { line: Some(1) }
                    }
                );
            };
        }

        macro_rules! instruction_assert_default {
            ( $inputB3D:literal, $inputCSV:literal, $args:literal, $data:expr, $default_args:literal, $default_data:expr ) => {
                instruction_assert!($inputB3D, $inputCSV, $args, $data);
                instruction_assert!($inputB3D, $inputCSV, $default_args, $default_data);
            };
        }

        #[test]
        fn mesh_builder() {
            instruction_assert!(
                "[meshbuilder]",
                "CreateMeshBuilder",
                "",
                InstructionData::CreateMeshBuilder
            );
        }

        #[test]
        fn add_vertex() {
            instruction_assert!(
                "Vertex",
                "AddVertex",
                "1, 2, 3, 4, 5, 6",
                InstructionData::AddVertex(AddVertex {
                    location: Vector3::new(1.0, 2.0, 3.0),
                    normal: Vector3::new(4.0, 5.0, 6.0),
                })
            );
        }

        #[test]
        fn add_face() {
            instruction_assert!(
                "Face",
                "AddFace",
                "1, 2, 3, 4, 5, 6",
                InstructionData::AddFace(AddFace {
                    indexes: vec![1, 2, 3, 4, 5, 6],
                    sides: Sides::One,
                })
            );
        }

        #[test]
        fn add_face2() {
            instruction_assert!(
                "Face2",
                "AddFace2",
                "1, 2, 3, 4, 5, 6",
                InstructionData::AddFace(AddFace {
                    indexes: vec![1, 2, 3, 4, 5, 6],
                    sides: Sides::Two,
                })
            );
        }

        #[test]
        fn cube() {
            instruction_assert!(
                "Cube",
                "Cube",
                "1, 2, 3",
                InstructionData::Cube(Cube {
                    half_dim: Vector3::new(1.0, 2.0, 3.0)
                })
            );
        }

        #[test]
        fn cylinder() {
            instruction_assert!(
                "Cylinder",
                "Cylinder",
                "1, 2, 3, 4",
                InstructionData::Cylinder(Cylinder {
                    sides: 1,
                    upper_radius: 2.0,
                    lower_radius: 3.0,
                    height: 4.0,
                })
            );
        }

        #[test]
        fn generate_normals() {
            no_instruction_assert!("GenerateNormals", "GenerateNormals", "");
        }

        #[test]
        fn texture() {
            no_instruction_assert!("[texture]", "Texture", "");
        }

        #[test]
        fn translate() {
            instruction_assert!(
                "Translate",
                "Translate",
                "1, 2, 3",
                InstructionData::Translate(Translate {
                    value: Vector3::new(1.0, 2.0, 3.0),
                    application: ApplyTo::SingleMesh,
                })
            );
        }

        #[test]
        fn translate_all() {
            instruction_assert!(
                "TranslateAll",
                "TranslateAll",
                "1, 2, 3",
                InstructionData::Translate(Translate {
                    value: Vector3::new(1.0, 2.0, 3.0),
                    application: ApplyTo::AllMeshes,
                })
            );
        }

        #[test]
        fn scale() {
            instruction_assert!(
                "Scale",
                "Scale",
                "1, 2, 3",
                InstructionData::Scale(Scale {
                    value: Vector3::new(1.0, 2.0, 3.0),
                    application: ApplyTo::SingleMesh,
                })
            );
        }

        #[test]
        fn scale_all() {
            instruction_assert!(
                "ScaleAll",
                "ScaleAll",
                "1, 2, 3",
                InstructionData::Scale(Scale {
                    value: Vector3::new(1.0, 2.0, 3.0),
                    application: ApplyTo::AllMeshes,
                })
            );
        }

        #[test]
        fn rotate() {
            instruction_assert!(
                "Rotate",
                "Rotate",
                "1, 2, 3, 4",
                InstructionData::Rotate(Rotate {
                    value: Vector3::new(1.0, 2.0, 3.0),
                    angle: 4.0,
                    application: ApplyTo::SingleMesh,
                })
            );
        }

        #[test]
        fn rotate_all() {
            instruction_assert!(
                "RotateAll",
                "RotateAll",
                "1, 2, 3, 4",
                InstructionData::Rotate(Rotate {
                    value: Vector3::new(1.0, 2.0, 3.0),
                    angle: 4.0,
                    application: ApplyTo::AllMeshes,
                })
            );
        }

        #[test]
        fn sheer() {
            instruction_assert!(
                "Sheer",
                "Sheer",
                "1, 2, 3, 4, 5, 6, 7",
                InstructionData::Sheer(Sheer {
                    direction: Vector3::new(1.0, 2.0, 3.0),
                    sheer: Vector3::new(4.0, 5.0, 6.0),
                    ratio: 7.0,
                    application: ApplyTo::SingleMesh,
                })
            );
        }

        #[test]
        fn sheer_all() {
            instruction_assert!(
                "SheerAll",
                "SheerAll",
                "1, 2, 3, 4, 5, 6, 7",
                InstructionData::Sheer(Sheer {
                    direction: Vector3::new(1.0, 2.0, 3.0),
                    sheer: Vector3::new(4.0, 5.0, 6.0),
                    ratio: 7.0,
                    application: ApplyTo::AllMeshes,
                })
            );
        }

        #[test]
        fn mirror() {
            instruction_assert!(
                "Mirror",
                "Mirror",
                "0, 1, 0",
                InstructionData::Mirror(Mirror {
                    directions: Vector3::new(false, true, false),
                    application: ApplyTo::SingleMesh,
                })
            );
        }

        #[test]
        fn mirror_all() {
            instruction_assert!(
                "MirrorAll",
                "MirrorAll",
                "0, 1, 0",
                InstructionData::Mirror(Mirror {
                    directions: Vector3::new(false, true, false),
                    application: ApplyTo::AllMeshes,
                })
            );
        }

        #[test]
        fn color() {
            instruction_assert!(
                "Color",
                "SetColor",
                "1, 2, 3, 4",
                InstructionData::SetColor(SetColor {
                    color: ColorU8RGBA::new(1, 2, 3, 4),
                })
            );
        }

        #[test]
        fn emmisive_color() {
            instruction_assert!(
                "EmissiveColor",
                "SetEmissiveColor",
                "1, 2, 3",
                InstructionData::SetEmissiveColor(SetEmissiveColor {
                    color: ColorU8RGB::new(1, 2, 3),
                })
            );
        }

        #[test]
        fn blend_mode() {
            instruction_assert!(
                "BlendMode",
                "SetBlendMode",
                "Additive, 2, DivideExponent2",
                InstructionData::SetBlendMode(SetBlendMode {
                    blend_mode: BlendMode::Additive,
                    glow_half_distance: 2,
                    glow_attenutation_mode: GlowAttenuationMode::DivideExponent2,
                })
            );
            instruction_assert!(
                "BlendMode",
                "SetBlendMode",
                "Additive, 3, DivideExponent4",
                InstructionData::SetBlendMode(SetBlendMode {
                    blend_mode: BlendMode::Additive,
                    glow_half_distance: 3,
                    glow_attenutation_mode: GlowAttenuationMode::DivideExponent4,
                })
            );
            instruction_assert!(
                "BlendMode",
                "SetBlendMode",
                "Normal, 2, DivideExponent2",
                InstructionData::SetBlendMode(SetBlendMode {
                    blend_mode: BlendMode::Normal,
                    glow_half_distance: 2,
                    glow_attenutation_mode: GlowAttenuationMode::DivideExponent2,
                })
            );
            instruction_assert!(
                "BlendMode",
                "SetBlendMode",
                "Normal, 3, DivideExponent4",
                InstructionData::SetBlendMode(SetBlendMode {
                    blend_mode: BlendMode::Normal,
                    glow_half_distance: 3,
                    glow_attenutation_mode: GlowAttenuationMode::DivideExponent4,
                })
            );
        }

        #[test]
        fn load_texture() {
            instruction_assert!(
                "Load",
                "LoadTexture",
                "path/day.png, path/night.png",
                InstructionData::LoadTexture(LoadTexture {
                    daytime: "path/day.png".into(),
                    nighttime: "path/night.png".into(),
                })
            );
        }

        #[test]
        fn decal_transparent_color() {
            instruction_assert!(
                "Transparent",
                "SetDecalTransparentColor",
                "1, 2, 3",
                InstructionData::SetDecalTransparentColor(SetDecalTransparentColor {
                    color: ColorU8RGB::new(1, 2, 3),
                })
            );
        }

        #[test]
        fn texture_coordinates() {
            instruction_assert!(
                "Coordinates",
                "SetTextureCoordinates",
                "1, 2, 3",
                InstructionData::SetTextureCoordinates(SetTextureCoordinates {
                    index: 1,
                    coords: Vector2::new(2.0, 3.0),
                })
            );
        }
    }
}
