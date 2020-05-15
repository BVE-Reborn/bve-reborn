use crate::parse::{
    mesh::{instructions::*, FileType, MeshError, MeshErrorKind, MeshWarning, MeshWarningKind},
    util::strip_comments,
    Span,
};
use csv::{ReaderBuilder, StringRecord, Trim};
use log::trace;
use std::iter::FromIterator;

/// Adds a comma after the first space on each line. Forces newline on last line. Lowercases string.
pub(in crate::parse::mesh::instructions) fn b3d_to_csv_syntax(input: &str) -> String {
    trace!("Processing .b3d into .csv");

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

enum DeserializeInstructionError {
    MeshError(MeshError),
    MeshWarning(MeshWarning),
}

impl From<MeshError> for DeserializeInstructionError {
    fn from(e: MeshError) -> Self {
        Self::MeshError(e)
    }
}

impl From<MeshWarning> for DeserializeInstructionError {
    fn from(e: MeshWarning) -> Self {
        Self::MeshWarning(e)
    }
}

impl From<csv::Error> for DeserializeInstructionError {
    fn from(e: csv::Error) -> Self {
        Self::MeshError(e.into())
    }
}

fn deserialize_instruction(
    inst_type: InstructionType,
    record: &StringRecord,
    span: Span,
) -> Result<Instruction, DeserializeInstructionError> {
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
            return Err(MeshWarning {
                kind: MeshWarningKind::UselessInstruction {
                    name: String::from("GenerateNormals"),
                },
                location: span,
            }
            .into());
        }
        InstructionType::Texture => {
            return Err(MeshWarning {
                kind: MeshWarningKind::UselessInstruction {
                    name: String::from("[texture]"),
                },
                location: span,
            }
            .into());
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
    trace!(
        "Creating instructions for {:#?} mesh of length {}",
        file_type,
        input.len()
    );
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

    let stripped = strip_comments(&processed, ';');

    let csv_reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(stripped.as_bytes());

    let mut instructions = InstructionList::new();
    'l: for line in csv_reader.into_records() {
        match line {
            Ok(record) => {
                // Get the line number
                let span: Span = record.position().into();
                // Parse the instruction name
                let instruction: InstructionType = match record.get(0) {
                    Some(name) => {
                        if let Ok(v) = serde_plain::from_str(name) {
                            v
                        } else {
                            // If only whitespace, this is an instance a line with just commmas `,,,,,`, ignore it
                            if !(name.chars().all(char::is_whitespace)) {
                                instructions.errors.push(MeshError {
                                    location: span,
                                    kind: MeshErrorKind::UnknownInstruction { name: name.to_owned() },
                                });
                            }
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
                    Err(DeserializeInstructionError::MeshWarning(mut e)) => {
                        e.location = span;
                        instructions.warnings.push(e)
                    }
                    Err(DeserializeInstructionError::MeshError(mut e)) => {
                        e.location = span;
                        instructions.errors.push(e)
                    }
                }
            }
            Err(_e) => {}
        }
    }

    instructions
}

#[cfg(test)]
mod test {
    use crate::{
        parse::{
            mesh::{instructions::*, FileType},
            Span,
        },
        BVec3, ColorU8RGB, ColorU8RGBA,
    };
    use glam::{Vec2, Vec3};

    use crate::parse::mesh::instructions::b3d_to_csv_syntax;

    #[bve_derive::bve_test]
    #[test]
    fn comma_add() {
        assert_eq!(b3d_to_csv_syntax("myinstruction arg1"), "myinstruction, arg1\n");
    }

    #[bve_derive::bve_test]
    #[test]
    fn multiline_comma_add() {
        assert_eq!(
            b3d_to_csv_syntax("myinstruction arg1\nmyother arg2"),
            "myinstruction, arg1\nmyother, arg2\n"
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn spaceless() {
        assert_eq!(b3d_to_csv_syntax("myinstruction"), "myinstruction\n");
        assert_eq!(b3d_to_csv_syntax(""), "\n");
    }

    #[bve_derive::bve_test]
    #[test]
    fn multiline_spaceless() {
        assert_eq!(b3d_to_csv_syntax("myinstruction\n\nfk2"), "myinstruction\n\nfk2\n");
        assert_eq!(b3d_to_csv_syntax("\n\n"), "\n\n");
    }

    macro_rules! no_instruction_assert_no_errors {
        ($inputB3D:literal, $inputCSV:literal, $args:literal) => {
            let result_a = create_instructions(concat!($inputB3D, " ", $args).into(), FileType::B3D);
            if !result_a.warnings.is_empty() {
                panic!("WARNINGS!! {:#?}", result_a)
            }
            if !result_a.errors.is_empty() {
                panic!("ERRORS!! {:#?}", result_a)
            }
            assert_eq!(result_a.instructions.len(), 0);
            let result_b = create_instructions(concat!($inputCSV, ",", $args).into(), FileType::CSV);
            if !result_b.warnings.is_empty() {
                panic!("WARNINGS!! {:#?}", result_b)
            }
            if !result_b.errors.is_empty() {
                panic!("ERRORS!! {:#?}", result_b)
            }
            assert_eq!(result_b.instructions.len(), 0);
        };
    }

    macro_rules! no_instruction_assert_warnings {
        ($inputB3D:literal, $inputCSV:literal, $args:literal) => {
            let result_a = create_instructions(concat!($inputB3D, " ", $args).into(), FileType::B3D);
            if result_a.warnings.is_empty() {
                panic!("Missing Warnings: {:#?}", result_a)
            }
            if !result_a.errors.is_empty() {
                panic!("ERRORS!! {:#?}", result_a)
            }
            assert_eq!(result_a.instructions.len(), 0);
            let result_b = create_instructions(concat!($inputCSV, ",", $args).into(), FileType::CSV);
            if result_b.warnings.is_empty() {
                panic!("Missing Warnings: {:#?}", result_b)
            }
            if !result_b.errors.is_empty() {
                panic!("ERRORS!! {:#?}", result_b)
            }
            assert_eq!(result_b.instructions.len(), 0);
        };
    }

    #[allow(unused_macros)]
    macro_rules! no_instruction_assert_errors {
        ($inputB3D:literal, $inputCSV:literal, $args:literal) => {
            let result_a = create_instructions(concat!($inputB3D, " ", $args).into(), FileType::B3D);
            if !result_a.warnings.is_empty() {
                panic!("WARNINGS!! {:#?}", result_a)
            }
            if result_a.errors.is_empty() {
                panic!("Missing Errors: {:#?}", result_a)
            }
            assert_eq!(result_a.instructions.len(), 0);
            let result_b = create_instructions(concat!($inputCSV, ",", $args).into(), FileType::CSV);
            if result_b.errors.is_empty() {
                panic!("Missing Errors: {:#?}", result_b)
            }
            if !result_b.warnings.is_empty() {
                panic!("WARNINGS!! {:#?}", result_b)
            }
            assert_eq!(result_b.instructions.len(), 0);
        };
    }

    macro_rules! instruction_assert {
        ($inputB3D:literal, $inputCSV:literal, $args:literal, $data:expr) => {
            let result_a = create_instructions(concat!($inputB3D, " ", $args).into(), FileType::B3D);
            if !result_a.warnings.is_empty() {
                panic!("WARNINGS!! {:#?}", result_a)
            }
            if !result_a.errors.is_empty() {
                panic!("ERRORS!! {:#?}", result_a)
            }
            assert_eq!(*result_a.instructions.get(0).unwrap(), Instruction {
                data: $data,
                span: Span::from_line(1),
            });
            let result_b = create_instructions(concat!($inputCSV, ",", $args).into(), FileType::CSV);
            if !result_b.warnings.is_empty() {
                panic!("WARNINGS!! {:#?}", result_b)
            }
            if !result_b.errors.is_empty() {
                panic!("ERRORS!! {:#?}", result_b)
            }
            assert_eq!(*result_b.instructions.get(0).unwrap(), Instruction {
                data: $data,
                span: Span::from_line(1),
            });
        };
    }

    macro_rules! instruction_assert_default {
        (
            $inputB3D:literal, $inputCSV:literal, $args:literal, $data:expr, $default_args:literal, $default_data:expr
        ) => {
            instruction_assert!($inputB3D, $inputCSV, $args, $data);
            instruction_assert!($inputB3D, $inputCSV, $default_args, $default_data);
        };
    }

    #[bve_derive::bve_test]
    #[test]
    fn empty_line() {
        let result_a = create_instructions("", FileType::B3D);
        assert_eq!(result_a.instructions.len(), 0);
        assert_eq!(result_a.errors.len(), 0);
        let result_b = create_instructions("", FileType::CSV);
        assert_eq!(result_b.instructions.len(), 0);
        assert_eq!(result_b.errors.len(), 0);
    }

    #[bve_derive::bve_test]
    #[test]
    fn empty_line_with_commas() {
        no_instruction_assert_no_errors!(",,,,,,", ",,,,,,,", "");
    }

    #[bve_derive::bve_test]
    #[test]
    fn no_arguments() {
        instruction_assert!(
            "Vertex",
            "AddVertex",
            "",
            InstructionData::AddVertex(AddVertex {
                position: Vec3::zero(),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn too_many_arguments() {
        instruction_assert!(
            "Vertex",
            "AddVertex",
            ",,,,,,,,,,,,,,,,,,,,,,,,,,,,,,",
            InstructionData::AddVertex(AddVertex {
                position: Vec3::zero(),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn too_many_arguments_end_vector() {
        instruction_assert!(
            "Face",
            "AddFace",
            "0, 1, 2, 3,",
            InstructionData::AddFace(AddFace {
                indexes: vec![0, 1, 2, 3],
                sides: Sides::One,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn too_many_arguments_middle_vector() {
        instruction_assert!(
            "Face",
            "AddFace",
            "0, 1, 2,,,,,,,,3",
            InstructionData::AddFace(AddFace {
                indexes: vec![0, 1, 2, 3],
                sides: Sides::One,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn beginning_of_line_comment() {
        no_instruction_assert_no_errors!(";", ";", "");
    }

    #[bve_derive::bve_test]
    #[test]
    fn middle_of_line_comment() {
        // Adapted from no_arguments
        instruction_assert!(
            "Vertex;",
            "AddVertex;",
            "1, 2, 3, 4, 5, 6", // these are commented out
            InstructionData::AddVertex(AddVertex {
                position: Vec3::zero(),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn end_of_line_comment() {
        // Adapted from no_arguments
        instruction_assert!(
            "Vertex",
            "AddVertex",
            "1, 2, 3, 4, 5, 6;",
            InstructionData::AddVertex(AddVertex {
                position: Vec3::new(1.0, 2.0, 3.0),
                normal: Vec3::new(4.0, 5.0, 6.0),
                texture_coord: Vec2::zero(),
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn mesh_builder() {
        instruction_assert!(
            "[meshbuilder]",
            "CreateMeshBuilder",
            "",
            InstructionData::CreateMeshBuilder(CreateMeshBuilder)
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn add_vertex() {
        instruction_assert_default!(
            "Vertex",
            "AddVertex",
            "1, 2, 3, 4, 5, 6",
            InstructionData::AddVertex(AddVertex {
                position: Vec3::new(1.0, 2.0, 3.0),
                normal: Vec3::new(4.0, 5.0, 6.0),
                texture_coord: Vec2::zero(),
            }),
            ",,,,,",
            InstructionData::AddVertex(AddVertex {
                position: Vec3::zero(),
                normal: Vec3::zero(),
                texture_coord: Vec2::zero(),
            })
        );
    }

    #[bve_derive::bve_test]
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

    #[bve_derive::bve_test]
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

    #[bve_derive::bve_test]
    #[test]
    fn cube() {
        instruction_assert_default!(
            "Cube",
            "Cube",
            "1, 2, 3",
            InstructionData::Cube(Cube {
                half_dim: Vec3::new(1.0, 2.0, 3.0)
            }),
            ",,",
            InstructionData::Cube(Cube {
                half_dim: Vec3::new(1.0, 1.0, 1.0)
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn cylinder() {
        instruction_assert_default!(
            "Cylinder",
            "Cylinder",
            "1, 2, 3, 4",
            InstructionData::Cylinder(Cylinder {
                sides: 1,
                upper_radius: 2.0,
                lower_radius: 3.0,
                height: 4.0,
            }),
            ",,,",
            InstructionData::Cylinder(Cylinder {
                sides: 8,
                upper_radius: 1.0,
                lower_radius: 1.0,
                height: 1.0,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn generate_normals() {
        no_instruction_assert_warnings!("GenerateNormals", "GenerateNormals", "");
    }

    #[bve_derive::bve_test]
    #[test]
    fn texture() {
        no_instruction_assert_warnings!("[texture]", "Texture", "");
    }

    #[bve_derive::bve_test]
    #[test]
    fn translate() {
        instruction_assert_default!(
            "Translate",
            "Translate",
            "1, 2, 3",
            InstructionData::Translate(Translate {
                value: Vec3::new(1.0, 2.0, 3.0),
                application: ApplyTo::SingleMesh,
            }),
            ",,",
            InstructionData::Translate(Translate {
                value: Vec3::zero(),
                application: ApplyTo::SingleMesh,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn translate_all() {
        instruction_assert_default!(
            "TranslateAll",
            "TranslateAll",
            "1, 2, 3",
            InstructionData::Translate(Translate {
                value: Vec3::new(1.0, 2.0, 3.0),
                application: ApplyTo::AllMeshes,
            }),
            ",,",
            InstructionData::Translate(Translate {
                value: Vec3::zero(),
                application: ApplyTo::AllMeshes,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn scale() {
        instruction_assert_default!(
            "Scale",
            "Scale",
            "1, 2, 3",
            InstructionData::Scale(Scale {
                value: Vec3::new(1.0, 2.0, 3.0),
                application: ApplyTo::SingleMesh,
            }),
            ",,",
            InstructionData::Scale(Scale {
                value: Vec3::one(),
                application: ApplyTo::SingleMesh,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn scale_all() {
        instruction_assert_default!(
            "ScaleAll",
            "ScaleAll",
            "1, 2, 3",
            InstructionData::Scale(Scale {
                value: Vec3::new(1.0, 2.0, 3.0),
                application: ApplyTo::AllMeshes,
            }),
            ",,",
            InstructionData::Scale(Scale {
                value: Vec3::one(),
                application: ApplyTo::AllMeshes,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn rotate() {
        instruction_assert_default!(
            "Rotate",
            "Rotate",
            "1, 2, 3, 4",
            InstructionData::Rotate(Rotate {
                axis: Vec3::new(1.0, 2.0, 3.0),
                angle: 4.0,
                application: ApplyTo::SingleMesh,
            }),
            ",,,",
            InstructionData::Rotate(Rotate {
                axis: Vec3::zero(),
                angle: 0.0,
                application: ApplyTo::SingleMesh,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn rotate_all() {
        instruction_assert_default!(
            "RotateAll",
            "RotateAll",
            "1, 2, 3, 4",
            InstructionData::Rotate(Rotate {
                axis: Vec3::new(1.0, 2.0, 3.0),
                angle: 4.0,
                application: ApplyTo::AllMeshes,
            }),
            ",,,",
            InstructionData::Rotate(Rotate {
                axis: Vec3::zero(),
                angle: 0.0,
                application: ApplyTo::AllMeshes,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn shear() {
        instruction_assert_default!(
            "Shear",
            "Shear",
            "1, 2, 3, 4, 5, 6, 7",
            InstructionData::Shear(Shear {
                direction: Vec3::new(1.0, 2.0, 3.0),
                shear: Vec3::new(4.0, 5.0, 6.0),
                ratio: 7.0,
                application: ApplyTo::SingleMesh,
            }),
            ",,,,,,",
            InstructionData::Shear(Shear {
                direction: Vec3::zero(),
                shear: Vec3::zero(),
                ratio: 0.0,
                application: ApplyTo::SingleMesh,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn shear_all() {
        instruction_assert_default!(
            "ShearAll",
            "ShearAll",
            "1, 2, 3, 4, 5, 6, 7",
            InstructionData::Shear(Shear {
                direction: Vec3::new(1.0, 2.0, 3.0),
                shear: Vec3::new(4.0, 5.0, 6.0),
                ratio: 7.0,
                application: ApplyTo::AllMeshes,
            }),
            ",,,,,,",
            InstructionData::Shear(Shear {
                direction: Vec3::zero(),
                shear: Vec3::zero(),
                ratio: 0.0,
                application: ApplyTo::AllMeshes,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn mirror() {
        instruction_assert_default!(
            "Mirror",
            "Mirror",
            "0, 1, 0",
            InstructionData::Mirror(Mirror {
                directions: BVec3::new(false, true, false),
                application: ApplyTo::SingleMesh,
            }),
            ",,",
            InstructionData::Mirror(Mirror {
                directions: BVec3::new(false, false, false),
                application: ApplyTo::SingleMesh,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn mirror_all() {
        instruction_assert_default!(
            "MirrorAll",
            "MirrorAll",
            "0, 1, 0",
            InstructionData::Mirror(Mirror {
                directions: BVec3::new(false, true, false),
                application: ApplyTo::AllMeshes,
            }),
            ",,",
            InstructionData::Mirror(Mirror {
                directions: BVec3::new(false, false, false),
                application: ApplyTo::AllMeshes,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn color() {
        instruction_assert_default!(
            "Color",
            "SetColor",
            "1, 2, 3, 4",
            InstructionData::SetColor(SetColor {
                color: ColorU8RGBA::new(1, 2, 3, 4),
            }),
            ",,,",
            InstructionData::SetColor(SetColor {
                color: ColorU8RGBA::splat(255),
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn emmisive_color() {
        instruction_assert_default!(
            "EmissiveColor",
            "SetEmissiveColor",
            "1, 2, 3",
            InstructionData::SetEmissiveColor(SetEmissiveColor {
                color: ColorU8RGB::new(1, 2, 3),
            }),
            ",,",
            InstructionData::SetEmissiveColor(SetEmissiveColor {
                color: ColorU8RGB::zero(),
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn blend_mode() {
        instruction_assert_default!(
            "BlendMode",
            "SetBlendMode",
            "Additive, 2, DivideExponent2",
            InstructionData::SetBlendMode(SetBlendMode {
                blend_mode: BlendMode::Additive,
                glow_half_distance: 2,
                glow_attenuation_mode: GlowAttenuationMode::DivideExponent2,
            }),
            ",,",
            InstructionData::SetBlendMode(SetBlendMode {
                blend_mode: BlendMode::Normal,
                glow_half_distance: 0,
                glow_attenuation_mode: GlowAttenuationMode::DivideExponent4,
            })
        );
        instruction_assert!(
            "BlendMode",
            "SetBlendMode",
            "Additive, 3, DivideExponent4",
            InstructionData::SetBlendMode(SetBlendMode {
                blend_mode: BlendMode::Additive,
                glow_half_distance: 3,
                glow_attenuation_mode: GlowAttenuationMode::DivideExponent4,
            })
        );
        instruction_assert!(
            "BlendMode",
            "SetBlendMode",
            "Normal, 2, DivideExponent2",
            InstructionData::SetBlendMode(SetBlendMode {
                blend_mode: BlendMode::Normal,
                glow_half_distance: 2,
                glow_attenuation_mode: GlowAttenuationMode::DivideExponent2,
            })
        );
        instruction_assert!(
            "BlendMode",
            "SetBlendMode",
            "Normal, 3, DivideExponent4",
            InstructionData::SetBlendMode(SetBlendMode {
                blend_mode: BlendMode::Normal,
                glow_half_distance: 3,
                glow_attenuation_mode: GlowAttenuationMode::DivideExponent4,
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn load_texture() {
        instruction_assert_default!(
            "Load",
            "LoadTexture",
            "path/day.png, path/night.png",
            InstructionData::LoadTexture(LoadTexture {
                daytime: "path/day.png".into(),
                nighttime: "path/night.png".into(),
            }),
            ",",
            InstructionData::LoadTexture(LoadTexture {
                daytime: String::new(),
                nighttime: String::new(),
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn instruction_assert_default() {
        instruction_assert_default!(
            "Transparent",
            "SetDecalTransparentColor",
            "1, 2, 3",
            InstructionData::SetDecalTransparentColor(SetDecalTransparentColor {
                color: ColorU8RGB::new(1, 2, 3),
            }),
            ",,",
            InstructionData::SetDecalTransparentColor(SetDecalTransparentColor {
                color: ColorU8RGB::zero(),
            })
        );
    }

    #[bve_derive::bve_test]
    #[test]
    fn texture_coordinates() {
        instruction_assert_default!(
            "Coordinates",
            "SetTextureCoordinates",
            "1, 2, 3",
            InstructionData::SetTextureCoordinates(SetTextureCoordinates {
                index: 1,
                coords: Vec2::new(2.0, 3.0),
            }),
            ",,",
            InstructionData::SetTextureCoordinates(SetTextureCoordinates {
                index: 0,
                coords: Vec2::zero(),
            })
        );
    }
}
