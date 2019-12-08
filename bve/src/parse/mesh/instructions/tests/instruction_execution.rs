use crate::parse::mesh::instructions::*;
use crate::parse::mesh::Span;
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
        0: AddVertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::from_value(0.0),
        },
        1: AddVertex {
            position: Vector3::new(-0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        2: AddVertex {
            position: Vector3::new(0.866025, 0.0, 0.5),
            normal: Vector3::from_value(0.0),
        },
        3: AddFace {
            indexes: vec![0, 1, 2],
            sides: Sides::One,
        },
        4: LoadTexture {
            daytime: String::from("day_tex"),
            nighttime: String::from("night_tex"),
        }
    );

    let result = execution::generate_meshes(v);
    let first_mesh = &result.meshes[0];
    assert_eq!(first_mesh.vertices[0].position, Vector3::new(0.0, 0.0, 0.0));
    assert_eq!(first_mesh.vertices[1].position, Vector3::new(-0.866025, 0.0, 0.5));
    assert_eq!(first_mesh.vertices[2].position, Vector3::new(0.866025, 0.0, 0.5));
}
