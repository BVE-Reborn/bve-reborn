use crate::parse::mesh::instructions::*;
use crate::parse::mesh::{FileType, Span};
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Vector2, Vector3};

macro_rules! no_instruction_assert {
    ( $inputB3D:literal, $inputCSV:literal, $args:literal ) => {
        let result_a = create_instructions(concat!($inputB3D, " ", $args).into(), FileType::B3D);
        if result_a.errors.is_empty() {
            panic!("Missing Errors: {:#?}", result_a)
        }
        assert_eq!(result_a.instructions.len(), 0);
        let result_b = create_instructions(concat!($inputCSV, ",", $args).into(), FileType::CSV);
        if result_b.errors.is_empty() {
            panic!("Missing Errors: {:#?}", result_b)
        }
        assert_eq!(result_b.instructions.len(), 0);
    };
}

macro_rules! instruction_assert {
    ( $inputB3D:literal, $inputCSV:literal, $args:literal, $data:expr ) => {
        let result_a = create_instructions(concat!($inputB3D, " ", $args).into(), FileType::B3D);
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
        let result_b = create_instructions(concat!($inputCSV, ",", $args).into(), FileType::CSV);
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
fn empty_line() {
    let result_a = create_instructions("", FileType::B3D);
    assert_eq!(result_a.instructions.len(), 0);
    assert_eq!(result_a.errors.len(), 0);
    let result_b = create_instructions("", FileType::CSV);
    assert_eq!(result_b.instructions.len(), 0);
    assert_eq!(result_b.errors.len(), 0);
}

#[test]
fn no_arguments() {
    instruction_assert!(
        "Vertex",
        "AddVertex",
        "",
        InstructionData::AddVertex(AddVertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            texture_coord: Vector2::new(0.0, 0.0),
        })
    );
}

#[test]
fn too_many_arguments() {
    instruction_assert!(
        "Vertex",
        "AddVertex",
        ",,,,,,,,,,,,,,,,,,,,,,,,,,,,,,",
        InstructionData::AddVertex(AddVertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            texture_coord: Vector2::new(0.0, 0.0),
        })
    );
}

#[test]
fn mesh_builder() {
    instruction_assert!(
        "[meshbuilder]",
        "CreateMeshBuilder",
        "",
        InstructionData::CreateMeshBuilder(CreateMeshBuilder)
    );
}

#[test]
fn add_vertex() {
    instruction_assert_default!(
        "Vertex",
        "AddVertex",
        "1, 2, 3, 4, 5, 6",
        InstructionData::AddVertex(AddVertex {
            position: Vector3::new(1.0, 2.0, 3.0),
            normal: Vector3::new(4.0, 5.0, 6.0),
            texture_coord: Vector2::new(0.0, 0.0),
        }),
        ",,,,,",
        InstructionData::AddVertex(AddVertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            texture_coord: Vector2::new(0.0, 0.0),
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
    instruction_assert_default!(
        "Translate",
        "Translate",
        "1, 2, 3",
        InstructionData::Translate(Translate {
            value: Vector3::new(1.0, 2.0, 3.0),
            application: ApplyTo::SingleMesh,
        }),
        ",,",
        InstructionData::Translate(Translate {
            value: Vector3::new(0.0, 0.0, 0.0),
            application: ApplyTo::SingleMesh,
        })
    );
}

#[test]
fn translate_all() {
    instruction_assert_default!(
        "TranslateAll",
        "TranslateAll",
        "1, 2, 3",
        InstructionData::Translate(Translate {
            value: Vector3::new(1.0, 2.0, 3.0),
            application: ApplyTo::AllMeshes,
        }),
        ",,",
        InstructionData::Translate(Translate {
            value: Vector3::new(0.0, 0.0, 0.0),
            application: ApplyTo::AllMeshes,
        })
    );
}

#[test]
fn scale() {
    instruction_assert_default!(
        "Scale",
        "Scale",
        "1, 2, 3",
        InstructionData::Scale(Scale {
            value: Vector3::new(1.0, 2.0, 3.0),
            application: ApplyTo::SingleMesh,
        }),
        ",,",
        InstructionData::Scale(Scale {
            value: Vector3::new(1.0, 1.0, 1.0),
            application: ApplyTo::SingleMesh,
        })
    );
}

#[test]
fn scale_all() {
    instruction_assert_default!(
        "ScaleAll",
        "ScaleAll",
        "1, 2, 3",
        InstructionData::Scale(Scale {
            value: Vector3::new(1.0, 2.0, 3.0),
            application: ApplyTo::AllMeshes,
        }),
        ",,",
        InstructionData::Scale(Scale {
            value: Vector3::new(1.0, 1.0, 1.0),
            application: ApplyTo::AllMeshes,
        })
    );
}

#[test]
fn rotate() {
    instruction_assert_default!(
        "Rotate",
        "Rotate",
        "1, 2, 3, 4",
        InstructionData::Rotate(Rotate {
            axis: Vector3::new(1.0, 2.0, 3.0),
            angle: 4.0,
            application: ApplyTo::SingleMesh,
        }),
        ",,,",
        InstructionData::Rotate(Rotate {
            axis: Vector3::new(0.0, 0.0, 0.0),
            angle: 0.0,
            application: ApplyTo::SingleMesh,
        })
    );
}

#[test]
fn rotate_all() {
    instruction_assert_default!(
        "RotateAll",
        "RotateAll",
        "1, 2, 3, 4",
        InstructionData::Rotate(Rotate {
            axis: Vector3::new(1.0, 2.0, 3.0),
            angle: 4.0,
            application: ApplyTo::AllMeshes,
        }),
        ",,,",
        InstructionData::Rotate(Rotate {
            axis: Vector3::new(0.0, 0.0, 0.0),
            angle: 0.0,
            application: ApplyTo::AllMeshes,
        })
    );
}

#[test]
fn shear() {
    instruction_assert_default!(
        "Shear",
        "Shear",
        "1, 2, 3, 4, 5, 6, 7",
        InstructionData::Shear(Shear {
            direction: Vector3::new(1.0, 2.0, 3.0),
            shear: Vector3::new(4.0, 5.0, 6.0),
            ratio: 7.0,
            application: ApplyTo::SingleMesh,
        }),
        ",,,,,,",
        InstructionData::Shear(Shear {
            direction: Vector3::new(0.0, 0.0, 0.0),
            shear: Vector3::new(0.0, 0.0, 0.0),
            ratio: 0.0,
            application: ApplyTo::SingleMesh,
        })
    );
}

#[test]
fn shear_all() {
    instruction_assert_default!(
        "ShearAll",
        "ShearAll",
        "1, 2, 3, 4, 5, 6, 7",
        InstructionData::Shear(Shear {
            direction: Vector3::new(1.0, 2.0, 3.0),
            shear: Vector3::new(4.0, 5.0, 6.0),
            ratio: 7.0,
            application: ApplyTo::AllMeshes,
        }),
        ",,,,,,",
        InstructionData::Shear(Shear {
            direction: Vector3::new(0.0, 0.0, 0.0),
            shear: Vector3::new(0.0, 0.0, 0.0),
            ratio: 0.0,
            application: ApplyTo::AllMeshes,
        })
    );
}

#[test]
fn mirror() {
    instruction_assert_default!(
        "Mirror",
        "Mirror",
        "0, 1, 0",
        InstructionData::Mirror(Mirror {
            directions: Vector3::new(false, true, false),
            application: ApplyTo::SingleMesh,
        }),
        ",,",
        InstructionData::Mirror(Mirror {
            directions: Vector3::new(false, false, false),
            application: ApplyTo::SingleMesh,
        })
    );
}

#[test]
fn mirror_all() {
    instruction_assert_default!(
        "MirrorAll",
        "MirrorAll",
        "0, 1, 0",
        InstructionData::Mirror(Mirror {
            directions: Vector3::new(false, true, false),
            application: ApplyTo::AllMeshes,
        }),
        ",,",
        InstructionData::Mirror(Mirror {
            directions: Vector3::new(false, false, false),
            application: ApplyTo::AllMeshes,
        })
    );
}

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
            color: ColorU8RGBA::new(255, 255, 255, 255),
        })
    );
}

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
            color: ColorU8RGB::new(0, 0, 0),
        })
    );
}

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
            color: ColorU8RGB::new(0, 0, 0),
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
