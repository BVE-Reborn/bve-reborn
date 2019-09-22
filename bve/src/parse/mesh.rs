use crate::{ColorU8RGBA, ColorU8RGB};
use cgmath::{Vector3, Vector2};
use indexmap::IndexSet;

mod instructions;

pub struct ParsedStaticObject {
    pub meshes: Vec<Mesh>,
    pub textures: TextureFileSet,
    pub errors: Vec<ParsingError>,
}

pub struct ParsingError {
    pub kind: ParsingErrorKind
}

pub enum ParsingErrorKind {

}

pub struct TextureFileSet {
    filenames: IndexSet<String>,
}

impl TextureFileSet {
    pub fn new() -> Self {
        TextureFileSet {
            filenames: IndexSet::new(),
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        TextureFileSet {
            filenames: IndexSet::with_capacity(size),
        }
    }

    pub fn add(&mut self, value: String) -> usize {
        self.filenames.insert_full(value).0
    }

    pub fn lookup(&self, idx: usize) -> Option<&str> {
        self.filenames.get_index(idx).map(|s| s.as_str())
    }

    pub fn merge(&mut self, other: TextureFileSet) {
        self.filenames.extend(other.filenames)
    }
}

pub struct Texture {
    pub texture_file: usize,
    pub decal_transparent_color: Option<ColorU8RGB>,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u64>,
    pub face_data: Vec<FaceData>,
    pub texture: Texture,
    pub color: ColorU8RGBA,
    pub blend_mode: BlendMode,
    pub glow: Glow,
}

#[repr(C)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub coord: Vector2<f32>
}

#[repr(C)]
pub struct FaceData {
    pub emission_color: ColorU8RGB,
}

pub struct Glow {
    pub attenuation_mode: GlowAttenuationMode,
    pub half_distance: u16,
}

#[repr(C)]
pub enum BlendMode {
    Normal,
    Additive,
}

#[repr(C)]
pub enum GlowAttenuationMode {
    DivideExponent2,
    DivideExponent4,
}
