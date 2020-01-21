use crate::parse::mesh::instructions::*;
use crate::parse::mesh::{FileType, MeshError, MeshErrorKind};
use crate::parse::Span;
use csv::{ReaderBuilder, StringRecord, Trim};
use std::iter::FromIterator;

/// Adds a comma after the first space on each line. Forces newline on last line. Lowercases string.
pub(in crate::parse::mesh::instructions) fn b3d_to_csv_syntax(input: &str) -> String {
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
) -> Result<Instruction, MeshError> {
    let data = match inst_type {
        InstructionType::CreateMeshBuilder => InstructionData::CreateMeshBuilder(CreateMeshBuilder),
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
            return Err(MeshError {
                kind: MeshErrorKind::UselessInstruction {
                    name: String::from("GenerateNormals"),
                },
                span,
            });
        }
        InstructionType::Texture => {
            return Err(MeshError {
                kind: MeshErrorKind::UselessInstruction {
                    name: String::from("[texture]"),
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
        InstructionType::Shear => {
            let mut parsed: Shear = record.deserialize(None)?;
            parsed.application = ApplyTo::SingleMesh;
            InstructionData::Shear(parsed)
        }
        InstructionType::ShearAll => {
            let mut parsed: Shear = record.deserialize(None)?;
            parsed.application = ApplyTo::AllMeshes;
            InstructionData::Shear(parsed)
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

/// Parse the given `input` as a `file_type` file and use it to generate an [`InstructionList`].
///
/// All errors are reported in [`InstructionList::errors`].
#[must_use]
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
                // Get the line number
                let span = record.position().into();
                // Parse the instruction name
                let instruction: InstructionType = match record.get(0) {
                    Some(name) => {
                        if let Ok(v) = serde_plain::from_str(name) {
                            v
                        } else {
                            instructions.errors.push(MeshError {
                                span,
                                kind: MeshErrorKind::UnknownInstruction { name: name.to_owned() },
                            });
                            continue 'l;
                        }
                    }
                    // Nothing in line
                    None => continue 'l,
                };

                // Remove the already parsed instruction name
                let arguments = StringRecord::from_iter(record.iter().skip(1));

                let inst = deserialize_instruction(instruction, &arguments, span);

                match inst {
                    Ok(i) => instructions.instructions.push(i),
                    Err(mut e) => {
                        e.span = span;
                        instructions.errors.push(e)
                    }
                }
            }
            Err(_e) => {}
        }
    }

    instructions
}
