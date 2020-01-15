use crate::parse::mesh::instructions::{Instruction, create_instructions, create_vertex, InstructionData, AddVertex};
use cgmath::{Vector3, Array, Vector2};
use obj::{Obj, SimplePolygon};
use std::io::{BufReader, Cursor};
use crate::parse::mesh::Span;

mod b3d_to_csv_syntax;
mod instruction_creation;
mod instruction_execution;

fn generate_instructions_from_obj(input: &'static [u8]) -> Vec<Instruction> {
    let mut input = input.to_vec();
    let mut buf = BufReader::new(Cursor::new(input));

    let obj: Obj<'_, SimplePolygon> = Obj::load_buf(&mut buf).expect("Unable to parse obj");
    let mut result = Vec::new();

    // For every face, we separately create the 4 vertices needed
    
    for position in &obj.po {
        result.push(Instruction{ span: Span {line: None}, data: InstructionData::AddVertex(AddVertex {
            position: Vector3::new(position[0], position[1], position[2]),
            normal: Vector3::from_value(0.0);
            texture_coord: Vector2::from_value(0.0);
        })});
    }

    unimplemented!()
}
