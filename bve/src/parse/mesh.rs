use crate::{ColorU8RGBA, ColorU8RGB};
use cgmath::{Vector3, Vector2};
use std::collections::HashMap;

pub struct TextureFileSet {
    data: Vec<String>,
    mapping: HashMap<String, usize>,
}

pub struct Texture {
    texture_file: usize,
    decal_transparent_color: Option<ColorU8RGB>,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u64>,
    face_data: Vec<FaceData>,
    texture: Texture,
    color: ColorU8RGBA,
    blend_mode: BlendMode,
    glow: Glow,
}

#[repr(C)]
pub struct Vertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    coord: Vector2<f32>
}

#[repr(C)]
pub struct FaceData {
    emission_color: ColorU8RGB,
}

#[repr(C)]
pub enum BlendMode {
    Normal,
    Additive,
}

pub struct Glow {
    attenuation_mode: GlowAttenuationMode,
    half_distance: u16,
}

#[repr(C)]
pub enum GlowAttenuationMode {
    DivideExponent2,
    DivideExponent4,
}
