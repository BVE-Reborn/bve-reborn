use crate::parse::mesh::instructions::*;
use crate::parse::mesh::{BlendMode, Glow, GlowAttenuationMode, Span};
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::Vector3;
use cgmath::{Array, Vector2};

macro_rules! generate_instruction_list {
    ($($num:literal: $id:ident { $($tokens:tt)* }),*) => {
        InstructionList {
            instructions: vec![$(Instruction{
                data: InstructionData::$id($id{
                    $($tokens)*
                }),
                span: Span{
                    line: Some($num)
                }
            }),+],
            errors: vec![],
        }
    };
}

#[test]
fn single_mesh() {
    let v = generate_instruction_list!(
        0: CreateMeshBuilder {},
        1: AddVertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::from_value(0.0),
        },
        2: AddVertex {
            position: Vector3::new(-0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        3: AddVertex {
            position: Vector3::new(0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        4: AddFace {
            indexes: vec![0, 1, 2],
            sides: Sides::One,
        },
        5: LoadTexture {
            daytime: String::from("day_tex"),
            nighttime: String::from("night_tex"),
        }
    );

    let result = execution::generate_meshes(v);
    assert_eq!(result.meshes.len(), 1);
    let mesh = &result.meshes[0];
    assert_eq!(mesh.vertices[0].position, Vector3::new(0.0, 0.0, 0.0));
    assert_eq!(mesh.vertices[1].position, Vector3::new(-0.866025, 0.0, 0.5));
    assert_eq!(mesh.vertices[2].position, Vector3::new(0.866025, 0.0, 0.5));
    for v in &mesh.vertices {
        assert_eq!(v.normal, Vector3::new(0.0, 1.0, 0.0));
    }
    for &v in &mesh.vertices {
        assert_eq!(v.coord, Vector2::from_value(0.0));
    }
    assert_eq!(mesh.indices, vec![0, 1, 2]);
    assert_eq!(mesh.blend_mode, BlendMode::Normal);
    assert_eq!(mesh.color, ColorU8RGBA::from_value(255));
    assert_eq!(
        mesh.glow,
        Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        }
    );
    assert_eq!(result.textures.len(), 1);
    assert_eq!(result.textures.lookup(0), Some("day_tex"));
    assert_eq!(result.errors.len(), 0);
}

/// First mesh uses all possible attributes besides texture coords,
/// helps test to make sure state is properly reset
#[test]
fn double_mesh() {
    let v = generate_instruction_list!(
        0: CreateMeshBuilder {},
        1: AddVertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::from_value(0.0),
        },
        2: AddVertex {
            position: Vector3::new(-0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        3: AddVertex {
            position: Vector3::new(0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        4: AddFace {
            indexes: vec![0, 1, 2],
            sides: Sides::One,
        },
        5: LoadTexture {
            daytime: String::from("day_tex"),
            nighttime: String::from("night_tex"),
        },
        6: SetBlendMode {
            blend_mode: BlendMode::Additive,
            glow_half_distance: 12u16,
            glow_attenuation_mode: GlowAttenuationMode::DivideExponent2,
        },
        7: SetEmissiveColor {
            color: ColorU8RGB::new(11, 12, 13)
        },
        8: SetColor {
            color: ColorU8RGBA::new(21, 22, 23, 24)
        },
        9: SetDecalTransparentColor {
            color: ColorU8RGB::new(31, 32, 33)
        },
        10: CreateMeshBuilder {},
        11: AddVertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::from_value(0.0),
        },
        12: AddVertex {
            position: Vector3::new(-0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        13: AddVertex {
            position: Vector3::new(0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        14: AddFace {
            indexes: vec![0, 1, 2],
            sides: Sides::One,
        },
        15: LoadTexture {
            daytime: String::from("other_day_tex"),
            nighttime: String::from("night_tex"),
        }
    );

    let result = execution::generate_meshes(v);
    assert_eq!(result.meshes.len(), 2);

    // First Mesh
    let mesh = &result.meshes[0];
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.vertices[0].position, Vector3::new(0.0, 0.0, 0.0));
    assert_eq!(mesh.vertices[1].position, Vector3::new(-0.866025, 0.0, 0.5));
    assert_eq!(mesh.vertices[2].position, Vector3::new(0.866025, 0.0, 0.5));
    for v in &mesh.vertices {
        assert_eq!(v.normal, Vector3::new(0.0, 1.0, 0.0));
    }
    for &v in &mesh.vertices {
        assert_eq!(v.coord, Vector2::from_value(0.0));
    }
    assert_eq!(mesh.indices, vec![0, 1, 2]);
    assert_eq!(mesh.blend_mode, BlendMode::Additive);
    assert_eq!(mesh.color, ColorU8RGBA::new(21, 22, 23, 24));
    assert_eq!(mesh.texture.emission_color, ColorU8RGB::new(11, 12, 13));
    assert_eq!(mesh.texture.decal_transparent_color, Some(ColorU8RGB::new(31, 32, 33)));
    assert_eq!(
        mesh.glow,
        Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent2,
            half_distance: 12,
        }
    );

    // Second Mesh
    let mesh = &result.meshes[1];
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.vertices[0].position, Vector3::new(0.0, 0.0, 0.0));
    assert_eq!(mesh.vertices[1].position, Vector3::new(-0.866025, 0.0, 0.5));
    assert_eq!(mesh.vertices[2].position, Vector3::new(0.866025, 0.0, 0.5));
    for v in &mesh.vertices {
        assert_eq!(v.normal, Vector3::new(0.0, 1.0, 0.0));
    }
    for &v in &mesh.vertices {
        assert_eq!(v.coord, Vector2::from_value(0.0));
    }
    assert_eq!(mesh.indices, vec![0, 1, 2]);
    assert_eq!(mesh.blend_mode, BlendMode::Normal);
    assert_eq!(mesh.color, ColorU8RGBA::from_value(255));
    assert_eq!(
        mesh.glow,
        Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        }
    );
    assert_eq!(result.textures.len(), 2);
    assert_eq!(result.textures.lookup(0), Some("day_tex"));
    assert_eq!(result.textures.lookup(1), Some("other_day_tex"));
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn texture_coords() {
    let v = generate_instruction_list!(
        0: CreateMeshBuilder {},
        1: AddVertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::from_value(0.0),
        },
        2: AddVertex {
            position: Vector3::new(-0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        3: AddVertex {
            position: Vector3::new(0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        4: AddFace {
            indexes: vec![0, 1, 2],
            sides: Sides::One,
        },
        5: LoadTexture {
            daytime: String::from("day_tex"),
            nighttime: String::from("night_tex"),
        },
        6: SetTextureCoordinates {
            index: 0,
            coords: Vector2::new(1.0, 1.0),
        },
        7: SetTextureCoordinates {
            index: 1,
            coords: Vector2::new(2.0, 2.0),
        },
        8: SetTextureCoordinates {
            index: 2,
            coords: Vector2::new(3.0, 3.0),
        }
    );

    let result = execution::generate_meshes(v);
    assert_eq!(result.meshes.len(), 1);
    let mesh = &result.meshes[0];
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.vertices[0].position, Vector3::new(0.0, 0.0, 0.0));
    assert_eq!(mesh.vertices[1].position, Vector3::new(-0.866025, 0.0, 0.5));
    assert_eq!(mesh.vertices[2].position, Vector3::new(0.866025, 0.0, 0.5));
    for v in &mesh.vertices {
        assert_eq!(v.normal, Vector3::new(0.0, 1.0, 0.0));
    }
    for (i, &v) in mesh.vertices.iter().enumerate() {
        assert_eq!(v.coord, Vector2::from_value((i + 1) as f32));
    }
    assert_eq!(mesh.indices, vec![0, 1, 2]);
    assert_eq!(mesh.blend_mode, BlendMode::Normal);
    assert_eq!(mesh.color, ColorU8RGBA::from_value(255));
    assert_eq!(
        mesh.glow,
        Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        }
    );
    assert_eq!(result.textures.len(), 1);
    assert_eq!(result.textures.lookup(0), Some("day_tex"));
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn cube() {
    let v = generate_instruction_list!(
        0: CreateMeshBuilder {},
        1: AddVertex {
            position: Vector3::new(1.000000, 1.000000, -1.000000),
            normal: Vector3::from_value(0.0),
        },
        2: AddVertex {
            position: Vector3::new(1.000000, -1.000000, -1.000000),
            normal: Vector3::from_value(0.0),
        },
        3: AddVertex {
            position: Vector3::new(1.000000, 1.000000, 1.000000),
            normal: Vector3::from_value(0.0),
        },
        4: AddVertex {
            position: Vector3::new(1.000000, -1.000000, 1.000000),
            normal: Vector3::from_value(0.0),
        },
        5: AddVertex {
            position: Vector3::new(-1.000000, 1.000000, -1.000000),
            normal: Vector3::from_value(0.0),
        },
        6: AddVertex {
            position: Vector3::new(-1.000000, -1.000000, -1.000000),
            normal: Vector3::from_value(0.0),
        },
        7: AddVertex {
            position: Vector3::new(-1.000000, 1.000000, 1.000000),
            normal: Vector3::from_value(0.0),
        },
        8: AddVertex {
            position: Vector3::new(-1.000000, -1.000000, 1.000000),
            normal: Vector3::from_value(0.0),
        },
        9: AddFace {
            indexes: vec![0, 4, 6, 2],
            sides: Sides::One,
        },
        10: AddFace {
            indexes: vec![3, 2, 6, 7],
            sides: Sides::One,
        },
        11: AddFace {
            indexes: vec![7, 6, 4, 5],
            sides: Sides::One,
        },
        12: AddFace {
            indexes: vec![5, 1, 3, 7],
            sides: Sides::One,
        },
        13: AddFace {
            indexes: vec![1, 0, 2, 3],
            sides: Sides::One,
        },
        14: AddFace {
            indexes: vec![5, 4, 0, 1],
            sides: Sides::One,
        },
        15: LoadTexture {
            daytime: String::from("day_tex"),
            nighttime: String::from("night_tex"),
        }
    );

    let result = execution::generate_meshes(v);
    assert_eq!(result.meshes.len(), 1);
    let mesh = &result.meshes[0];
    assert_eq!(mesh.vertices.len(), 24);
    assert_eq!(mesh.vertices[0].position, Vector3::new(0.0, 0.0, 0.0));
    assert_eq!(mesh.vertices[1].position, Vector3::new(-0.866025, 0.0, 0.5));
    assert_eq!(mesh.vertices[2].position, Vector3::new(0.866025, 0.0, 0.5));
    for v in &mesh.vertices {
        assert_eq!(v.normal, Vector3::new(0.0, 1.0, 0.0));
    }
    for &v in &mesh.vertices {
        assert_eq!(v.coord, Vector2::from_value(0.0));
    }
    assert_eq!(mesh.indices, vec![0, 1, 2]);
    assert_eq!(mesh.blend_mode, BlendMode::Normal);
    assert_eq!(mesh.color, ColorU8RGBA::from_value(255));
    assert_eq!(
        mesh.glow,
        Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        }
    );
    assert_eq!(result.textures.len(), 1);
    assert_eq!(result.textures.lookup(0), Some("day_tex"));
    assert_eq!(result.errors.len(), 0);
}
