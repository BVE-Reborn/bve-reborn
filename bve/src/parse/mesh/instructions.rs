use crate::parse::mesh::{Error, FileType, Span};
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

pub struct AddVertexMacro {
    pub location: cgmath::Vector3<f32>,
    pub normal: Vector3<f32>,
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
    pub upper_rad: f32,
    pub lower_rad: f32,
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
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_vector_proxy]
pub struct Sheer {
    pub direction: Vector3<f32>,
    pub sheer: Vector3<f32>,
    #[serde(skip)]
    pub application: ApplyTo,
}

#[bve_derive::serde_vector_proxy]
pub struct Mirror {
    pub directions: Vector3<bool>,
    #[serde(skip)]
    pub application: ApplyTo,
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
    pub glow_mode: GlowAttenuationMode,
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
            Err(e) => {}
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
        use crate::parse::mesh::instructions::{create, AddFace, AddVertex, Instruction, InstructionData, Sides};
        use crate::parse::mesh::{FileType, Span};
        use cgmath::Vector3;

        macro_rules! single_line_instruction_assert {
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
                    *result_a.instructions.get(0).unwrap(),
                    Instruction {
                        data: $data,
                        span: Span { line: Some(1) }
                    }
                );
            };
        }

        #[test]
        fn mesh_builder() {
            single_line_instruction_assert!(
                "[meshbuilder]",
                "CreateMeshBuilder",
                "",
                InstructionData::CreateMeshBuilder
            );
        }

        #[test]
        fn add_vertex() {
            single_line_instruction_assert!(
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
            single_line_instruction_assert!(
                "Face",
                "AddFace",
                "1, 2, 3, 4, 5, 6",
                InstructionData::AddFace(AddFace {
                    indexes: vec![1, 2, 3, 4, 5, 6],
                    sides: Sides::One,
                })
            );
        }
    }
}
