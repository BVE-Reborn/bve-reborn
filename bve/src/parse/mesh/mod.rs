//! B3D/CSV Static Meshes
//!
//! There are two ways to make a mesh from a file. First is to directly
//! call [`mesh_from_str`]. This is often the easiest as it takes care of
//! parsing, post processing, and execution automatically. The other way is by
//! manually calling the functions in [`instructions`].
//!
//! There is currently no way to stream from disk but these files are so small
//! who cares.

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
    /// No comma after instruction name
    B3D,
    /// Comma after instruction name
    CSV,
}

/// A single static object.
///
/// Despite having many meshes, the game treats this as a single game object, and always moves and behaves as a single
/// compound mesh.
///
/// Each mesh has it's own legacy properties that are on a per-mesh basis, so the meshes can't be combined.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParsedStaticObject {
    /// All meshes in the object.
    pub meshes: Vec<Mesh>,
    /// The set of texture names needed by mesh
    pub textures: TextureSet,
    /// Errors when creating the mesh. If there are enough errors, there might not even be any meshes!
    pub errors: Vec<MeshError>,
}

/// Set of texture filenames.
///
/// Based on [`IndexSet`] so is indexable by both filenames and by index. Does not allow duplicate items.
#[derive(Debug, Clone, PartialEq)]
pub struct TextureSet {
    filenames: IndexSet<String>,
}

impl TextureSet {
    /// Create a new `TextureSet` with no texture filenames in it.
    #[must_use]
    pub fn new() -> Self {
        Self {
            filenames: IndexSet::new(),
        }
    }

    /// Create a new `TextureSet` with a specific capacity
    #[must_use]
    pub fn with_capacity(size: usize) -> Self {
        Self {
            filenames: IndexSet::with_capacity(size),
        }
    }

    /// Texture count inside the set
    #[must_use]
    pub fn len(&self) -> usize {
        self.filenames.len()
    }

    /// Returns if the `TextureSet` is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.filenames.is_empty()
    }

    /// Add a new texture filenames to the set.
    ///
    /// Returns the index into the set of the added item.
    ///
    /// If the item already exists, returns the index of the existing item.
    pub fn add(&mut self, value: &str) -> usize {
        self.filenames.insert_full(value.into()).0
    }

    /// Lookup a texture filename by index.  
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

/// Static Object's reference to texture by filename
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    /// Index to get the texture name in Object's [`TextureSet`]
    pub texture_id: Option<usize>,
    pub decal_transparent_color: Option<ColorU8RGB>,
    pub emission_color: ColorU8RGB,
}

/// A mesh corresponds to a single `CreateMeshBuilder` and contains
/// all per-mesh data for it.
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
    #[inline(never)]
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

/// Parse the given `input` as `file_type` and generate a static object from it.
#[must_use]
#[bve_derive::span(WARN, "Load Mesh", ?file_type)]
pub fn mesh_from_str(input: &str, file_type: FileType) -> ParsedStaticObject {
    let instructions = instructions::create_instructions(input, file_type);
    instructions::generate_meshes(post_process(instructions))
}
