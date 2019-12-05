use crate::parse::mesh::instructions::*;
use crate::parse::mesh::Span;
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
            position: Vector3::new(0.0, 1.0, 1.0),
            normal: Vector3::new(0.0, 1.0, 1.0),
        },
        1: AddVertex {
            position: Vector3::new(0.0, 2.0, 2.0),
            normal: Vector3::new(0.0, 2.0, 2.0),
        },
        2: AddVertex {
            position: Vector3::new(0.0, 3.0, 3.0),
            normal: Vector3::new(0.0, 3.0, 3.0),
        },
        3: AddFace {
            indexes: vec![0, 1, 2],
            sides: Sides::One,
        }
    );

    let _result = execution::generate_meshes(v);
    dbg!(_result);
}
