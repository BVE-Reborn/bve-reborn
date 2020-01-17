//! B3D/CSV Static Meshes

use crate::parse::mesh::instructions::post_process;
use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Array, Vector2, Vector3};
pub use errors::*;
use indexmap::IndexSet;
use serde::Deserialize;

mod errors;
pub mod instructions;

/// Which type of file to parse as a mesh.
///
/// The differences are only if there is a comma after the instruction name, instructions from both will work as
/// expected.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileType {
    B3D,
    CSV,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParsedStaticObject {
    pub meshes: Vec<Mesh>,
    pub textures: TextureSet,
    pub errors: Vec<MeshError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureSet {
    filenames: IndexSet<String>,
}

impl TextureSet {
    #[must_use]
    pub fn new() -> Self {
        Self {
            filenames: IndexSet::new(),
        }
    }

    #[must_use]
    pub fn with_capacity(size: usize) -> Self {
        Self {
            filenames: IndexSet::with_capacity(size),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.filenames.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.filenames.is_empty()
    }

    pub fn add(&mut self, value: &str) -> usize {
        self.filenames.insert_full(value.into()).0
    }

    #[must_use]
    pub fn lookup(&self, idx: usize) -> Option<&str> {
        self.filenames.get_index(idx).map(std::string::String::as_str)
    }
}

impl Default for TextureSet {
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    /// Index into the according texture set
    pub texture_id: Option<usize>,
    pub decal_transparent_color: Option<ColorU8RGB>,
    pub emission_color: ColorU8RGB,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<usize>,
    pub texture: Texture,
    pub color: ColorU8RGBA,
    pub blend_mode: BlendMode,
    pub glow: Glow,
}

fn default_mesh() -> Mesh {
    Mesh {
        vertices: vec![],
        indices: vec![],
        texture: Texture {
            texture_id: None,
            emission_color: ColorU8RGB::from_value(0),
            decal_transparent_color: None,
        },
        color: ColorU8RGBA::from_value(255),
        blend_mode: BlendMode::Normal,
        glow: Glow {
            attenuation_mode: GlowAttenuationMode::DivideExponent4,
            half_distance: 0,
        },
    }
}

/// All per-vertex data in a BVE mesh
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub coord: Vector2<f32>,
    pub double_sided: bool,
}

impl Vertex {
    /// Debugging code that can fuck off in other situations
    #[allow(dead_code, clippy::use_debug, clippy::print_stdout)]
    fn print_positions(vertices: &[Self], indices: &[usize]) {
        println!("Vertices: [");
        for (i, v) in vertices.iter().enumerate() {
            println!("\t{}: [{}, {}, {}],", i, v.position.x, v.position.y, v.position.z);
        }
        println!("]");
        println!("{:?}", indices);
    }

    #[must_use]
    pub const fn from_position_normal_coord(position: Vector3<f32>, normal: Vector3<f32>, coord: Vector2<f32>) -> Self {
        Self {
            position,
            normal,
            coord,
            double_sided: false,
        }
    }

    #[must_use]
    pub fn from_position_normal(position: Vector3<f32>, normal: Vector3<f32>) -> Self {
        Self {
            position,
            normal,
            coord: Vector2::from_value(0.0),
            double_sided: false,
        }
    }

    #[must_use]
    pub fn from_position(position: Vector3<f32>) -> Self {
        Self {
            position,
            normal: Vector3::from_value(0.0),
            coord: Vector2::from_value(0.0),
            double_sided: false,
        }
    }
}

/// The glow numbers to use for this mesh. Not sure how exactly this works.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Glow {
    pub attenuation_mode: GlowAttenuationMode,
    pub half_distance: u16,
}

/// The blending mode to use when rendering the mesh
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    Normal,
    Additive,
}

/// No idea what this does, but every mesh has one or the other.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlowAttenuationMode {
    DivideExponent2,
    DivideExponent4,
}

#[must_use]
pub fn mesh_from_str(input: &str, file_type: FileType) -> ParsedStaticObject {
    let instructions = instructions::create_instructions(input, file_type);
    instructions::generate_meshes(post_process(instructions))
}
