#![allow(clippy::shadow_unrelated)] // Useful for testing

use crate::parse::mesh::instructions::*;
use crate::parse::mesh::*;
use crate::parse::Span;
use cgmath::{Array, Vector2, Vector3};
use obj::{Obj, SimplePolygon};
use std::io::{BufReader, Cursor};
use std::iter::FromIterator;

mod b3d_to_csv_syntax;
mod instruction_creation;
mod instruction_execution;
mod meshes;

fn generate_instructions_from_obj(input: &'static str) -> InstructionList {
    let input = input.as_bytes().to_vec();
    let mut buf = BufReader::new(Cursor::new(input));

    let obj: Obj<'_, SimplePolygon> = Obj::load_buf(&mut buf).expect("Unable to parse obj");
    let mut result = vec![Instruction {
        span: Span { line: None },
        data: InstructionData::CreateMeshBuilder(CreateMeshBuilder),
    }];

    // For every face, we separately create the vertices needed
    let mut index_count = 0;
    for face in &obj.objects[0].groups[0].polys {
        for (offset, vert) in face.iter().enumerate() {
            let position = obj.position[vert.0];
            let position: Vector3<f32> = Vector3::new(position[0], position[1], position[2]);
            let normal = obj.normal[vert.2.expect("OBJ must have normals")];
            let normal: Vector3<f32> = Vector3::new(normal[0], normal[1], normal[2]);
            result.push(Instruction {
                span: Span { line: None },
                data: InstructionData::AddVertex(AddVertex {
                    position,
                    normal,
                    texture_coord: Vector2::from_value(0.0),
                }),
            });
            let texture_coord = obj.texture[vert.1.expect("OBJ must have texture coords")];
            let texture_coord: Vector2<f32> = Vector2::new(texture_coord[0], texture_coord[1]);
            result.push(Instruction {
                span: Span { line: None },
                data: InstructionData::SetTextureCoordinates(SetTextureCoordinates {
                    coords: texture_coord,
                    index: index_count + offset,
                }),
            });
        }
        let face_vertices = face.len();
        result.push(Instruction {
            span: Span { line: None },
            data: InstructionData::AddFace(AddFace {
                indexes: Vec::from_iter(index_count..(index_count + face_vertices)),
                sides: Sides::One,
            }),
        });
        index_count += face_vertices;
    }

    InstructionList {
        instructions: result,
        errors: Vec::default(),
    }
}

/// Support code for automated mesh-mesh comparisons that I don't currently support
#[allow(dead_code)]
fn generate_mesh_from_obj(input: &'static str) -> Mesh {
    let input = input.as_bytes().to_vec();
    let mut buf = BufReader::new(Cursor::new(input));

    let obj: Obj<'_, SimplePolygon> = Obj::load_buf(&mut buf).expect("Unable to parse obj");
    let mut result = default_mesh();

    // For every face, we separately create the vertices needed
    let mut index_count = 0;
    for face in &obj.objects[0].groups[0].polys {
        for vert in face {
            let position = obj.position[vert.0];
            let position: Vector3<f32> = Vector3::new(position[0], position[1], position[2]);
            let normal = obj.normal[vert.2.expect("OBJ must have normals")];
            let normal: Vector3<f32> = Vector3::new(normal[0], normal[1], normal[2]);
            let texture_coord = obj.texture[vert.1.expect("OBJ must have texture coords")];
            let texture_coord: Vector2<f32> = Vector2::new(texture_coord[0], texture_coord[1]);
            result.vertices.push(Vertex {
                position,
                normal,
                coord: texture_coord,
                double_sided: false,
            });
        }
        let face_vertices = face.len();
        result.indices.extend(index_count..(index_count + face_vertices));
        index_count += face_vertices;
    }

    result
}
