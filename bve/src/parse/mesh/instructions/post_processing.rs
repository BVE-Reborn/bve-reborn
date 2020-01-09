use crate::parse::mesh::instructions::*;
use crate::parse::mesh::{MeshError, MeshErrorKind, Span};
use cgmath::{Array, Vector2, Vector3};
use std::f32::consts::PI;

pub fn post_process(mut instructions: InstructionList) -> InstructionList {
    let mut output = Vec::new();
    let meshes = instructions
        .instructions
        .split(|i| i.data == InstructionData::CreateMeshBuilder(CreateMeshBuilder));
    for mesh in meshes {
        let mesh = process_compound(mesh);
        let mesh = merge_texture_coords(&mesh, &mut instructions.errors);
        output.push(Instruction {
            span: Span { line: None },
            data: InstructionData::CreateMeshBuilder(CreateMeshBuilder),
        });
        output.extend(mesh);
    }

    instructions.instructions = output;

    instructions
}

/// Creates a AddVertex instruction from a position.
fn create_vertex(original: &Instruction, position: Vector3<f32>) -> Instruction {
    Instruction {
        span: original.span,
        data: InstructionData::AddVertex(AddVertex {
            position,
            normal: Vector3::from_value(0.0),
            texture_coord: Vector2::from_value(0.0),
        }),
    }
}

/// Creates AddFace instruction from an index list.
fn create_face(original: &Instruction, indexes: Vec<usize>) -> Instruction {
    Instruction {
        span: original.span,
        data: InstructionData::AddFace(AddFace {
            indexes,
            sides: Sides::One,
        }),
    }
}

/// For each the mesh given, replaces Cube and Cylinder commands with the appropriate AddVertex and AddFace commands.
fn process_compound(mesh: &[Instruction]) -> Vec<Instruction> {
    let mut result = Vec::new();

    // Need to keep track of the current vertex index so cubes and cylinders can use the correct indices
    let mut vertex_index = 0;
    for instruction in mesh {
        match &instruction.data {
            InstructionData::AddVertex(..) => {
                result.push(instruction.clone());
                vertex_index += 1;
            }
            InstructionData::Cube(cube) => {
                // http://openbve-project.net/documentation/HTML/object_cubecylinder.html

                let x = cube.half_dim.x;
                let y = cube.half_dim.y;
                let z = cube.half_dim.z;

                result.push(create_vertex(&instruction, Vector3::new(x, y, -z)));
                result.push(create_vertex(&instruction, Vector3::new(x, -y, -z)));
                result.push(create_vertex(&instruction, Vector3::new(-x, -y, -z)));
                result.push(create_vertex(&instruction, Vector3::new(-x, y, -z)));
                result.push(create_vertex(&instruction, Vector3::new(x, y, z)));
                result.push(create_vertex(&instruction, Vector3::new(x, -y, z)));
                result.push(create_vertex(&instruction, Vector3::new(-x, -y, z)));
                result.push(create_vertex(&instruction, Vector3::new(-x, y, z)));

                let vi = vertex_index;

                result.push(create_face(&instruction, vec![vi + 0, vi + 1, vi + 2, vi + 3]));
                result.push(create_face(&instruction, vec![vi + 0, vi + 4, vi + 5, vi + 1]));
                result.push(create_face(&instruction, vec![vi + 0, vi + 3, vi + 7, vi + 4]));
                result.push(create_face(&instruction, vec![vi + 6, vi + 5, vi + 4, vi + 7]));
                result.push(create_face(&instruction, vec![vi + 6, vi + 7, vi + 3, vi + 2]));
                result.push(create_face(&instruction, vec![vi + 6, vi + 2, vi + 1, vi + 5]));

                vertex_index += 8;
            }
            InstructionData::Cylinder(cylinder) => {
                // http://openbve-project.net/documentation/HTML/object_cubecylinder.html

                // Convert args to format used in above documentation
                let n = cylinder.sides;
                let n_f32 = n as f32;
                let r1 = cylinder.upper_radius;
                let r2 = cylinder.lower_radius;
                let h = cylinder.height;

                // Vertices
                for i in (0..n).map(|i| i as f32) {
                    let trig_arg = (2.0 * PI * i) / n_f32;
                    let cos = trig_arg.cos();
                    let sin = trig_arg.sin();
                    result.push(create_vertex(&instruction, Vector3::new(cos * r1, h / 2.0, sin * r1)));
                    result.push(create_vertex(&instruction, Vector3::new(cos * r2, -h / 2.0, sin * r2)));
                }

                // Faces
                let vi = vertex_index;

                let split = (n - 1).max(0) as usize;
                for i in 0..split {
                    result.push(create_face(
                        &instruction,
                        vec![vi + (2 * i + 2), vi + (2 * i + 3), vi + (2 * i + 1), vi + (2 * i + 0)],
                    ));
                    result.push(create_face(
                        &instruction,
                        vec![vi + 0, vi + 1, vi + (2 * i + 1), vi + (2 * i + 0)],
                    ));
                }
                vertex_index += (2 * n) as usize;
            }
            _ => {
                result.push(instruction.clone());
            }
        }
    }

    result
}

/// For each mesh give, fold the SetTextureCoordinates into the AddVertex commands
fn merge_texture_coords(mesh: &[Instruction], errors: &mut Vec<MeshError>) -> Vec<Instruction> {
    let mut result = Vec::new();
    // The instruction where the vertex index n is created is at result[vertex_indices[n]]
    let mut vertex_indices = Vec::new();

    for instruction in mesh {
        match &instruction.data {
            InstructionData::AddVertex(..) => {
                // Add the index for this vertex so it can be found again
                vertex_indices.push(result.len());
                result.push(instruction.clone());
            }
            InstructionData::SetTextureCoordinates(data) => {
                // Issue error if the index is out of range
                if data.index >= vertex_indices.len() {
                    errors.push(MeshError {
                        span: instruction.span,
                        kind: MeshErrorKind::OutOfBounds { idx: data.index },
                    });
                    continue;
                }
                // Go and set the texture coord of the AddVertex command
                // Unless there's a bug in the code, this is guaranteed to be a AddVertex.
                match &mut result[vertex_indices[data.index]].data {
                    InstructionData::AddVertex(vert) => {
                        vert.texture_coord = data.coords;
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                result.push(instruction.clone());
            }
        }
    }

    result
}
