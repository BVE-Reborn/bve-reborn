use crate::parse::mesh::instructions::*;
use crate::parse::mesh::{BlendMode, Glow, GlowAttenuationMode, Span};
use crate::ColorU8RGBA;
use cgmath::Array;
use cgmath::Vector3;

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
}
