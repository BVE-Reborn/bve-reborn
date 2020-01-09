//! B3D/CSV Static Meshes

use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Array, Vector2, Vector3};
pub use errors::*;
use indexmap::IndexSet;
use serde::Deserialize;

mod errors;
pub mod instructions;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileType {
    B3D,
    CSV,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParsedStaticObject {
    pub meshes: Vec<Mesh>,
    pub textures: TextureFileSet,
    pub errors: Vec<MeshError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureFileSet {
    filenames: IndexSet<String>,
}

impl TextureFileSet {
    pub fn new() -> Self {
        Self {
            filenames: IndexSet::new(),
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            filenames: IndexSet::with_capacity(size),
        }
    }

    pub fn len(&self) -> usize {
        self.filenames.len()
    }

    pub fn add(&mut self, value: &str) -> usize {
        self.filenames.insert_full(value.into()).0
    }

    pub fn lookup(&self, idx: usize) -> Option<&str> {
        self.filenames.get_index(idx).map(std::string::String::as_str)
    }
}

impl Default for TextureFileSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
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

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub coord: Vector2<f32>,
    pub double_sided: bool,
}

impl Vertex {
    fn from_position_normal(position: Vector3<f32>, normal: Vector3<f32>) -> Self {
        Self {
            position,
            normal,
            coord: Vector2::from_value(0.0),
            double_sided: false,
        }
    }
    fn from_position(position: Vector3<f32>) -> Self {
        Self {
            position,
            normal: Vector3::from_value(0.0),
            coord: Vector2::from_value(0.0),
            double_sided: false,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Glow {
    pub attenuation_mode: GlowAttenuationMode,
    pub half_distance: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    Normal,
    Additive,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlowAttenuationMode {
    DivideExponent2,
    DivideExponent4,
}

pub fn create_mesh_from_str(input: &str, file_type: FileType) -> ParsedStaticObject {
    let instructions = instructions::create_instructions(input, file_type);
    instructions::generate_meshes(instructions)
}
