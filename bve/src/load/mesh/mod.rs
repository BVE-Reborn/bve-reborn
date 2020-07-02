use crate::{
    filesystem::read_convert_utf8,
    load::mesh::execution::generate_meshes,
    parse::mesh::{
        instructions::{create_instructions, post_process},
        BlendMode, FileType, Glow, GlowAttenuationMode, MeshError, MeshWarning,
    },
    ColorU8RGB, ColorU8RGBA,
};
use async_std::path::Path;
pub use execution::*;
use glam::{Vec2, Vec3A};
use indexmap::IndexSet;
use std::{ffi::OsStr, ops::Deref};

mod execution;

/// A single static object.
///
/// Despite having many meshes, the game treats this as a single game object, and always moves and behaves as a single
/// compound mesh.
///
/// Each mesh has it's own legacy properties that are on a per-mesh basis, so the meshes can't be combined.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LoadedStaticMesh {
    /// All meshes in the object.
    pub meshes: Vec<Mesh>,
    /// The set of texture names needed by mesh
    pub textures: TextureSet,
    /// Warnings when creating the mesh. Does not affect mesh.
    pub warnings: Vec<MeshWarning>,
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

impl Deref for TextureSet {
    type Target = IndexSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.filenames
    }
}

impl IntoIterator for TextureSet {
    type IntoIter = <IndexSet<String> as IntoIterator>::IntoIter;
    type Item = String;

    fn into_iter(self) -> Self::IntoIter {
        self.filenames.into_iter()
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
    /// This data is duplicated inside the vertices, but is preserved for easy access
    pub color: ColorU8RGBA,
    pub blend_mode: BlendMode,
    pub glow: Glow,
}

#[must_use]
pub const fn default_mesh() -> Mesh {
    Mesh {
        vertices: vec![],
        indices: vec![],
        texture: Texture {
            texture_id: None,
            emission_color: ColorU8RGB::splat(0),
            decal_transparent_color: None,
        },
        color: ColorU8RGBA::splat(255),
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
    pub position: Vec3A,
    pub normal: Vec3A,
    pub color: ColorU8RGBA,
    pub coord: Vec2,
    pub double_sided: bool,
}

impl Vertex {
    /// Debugging code that can fuck off in other situations
    #[allow(dead_code, clippy::use_debug, clippy::print_stdout)]
    #[inline(never)]
    fn print_positions(vertices: &[Self], indices: &[usize]) {
        println!("Vertices: [");
        for (i, v) in vertices.iter().enumerate() {
            println!("\t{}: [{}, {}, {}],", i, v.position.x(), v.position.y(), v.position.z());
        }
        println!("]");
        println!("{:?}", indices);
    }

    #[must_use]
    pub const fn from_position_normal_coord(position: Vec3A, normal: Vec3A, coord: Vec2) -> Self {
        Self {
            position,
            normal,
            coord,
            color: ColorU8RGBA::splat(255),
            double_sided: false,
        }
    }

    #[must_use]
    pub fn from_position_normal(position: Vec3A, normal: Vec3A) -> Self {
        Self {
            position,
            normal,
            coord: Vec2::zero(),
            color: ColorU8RGBA::splat(255),
            double_sided: false,
        }
    }

    #[must_use]
    pub fn from_position(position: Vec3A) -> Self {
        Self {
            position,
            normal: Vec3A::zero(),
            coord: Vec2::zero(),
            color: ColorU8RGBA::splat(255),
            double_sided: false,
        }
    }
}

pub async fn load_mesh_from_file(file: impl AsRef<Path>) -> Option<LoadedStaticMesh> {
    let path = file.as_ref();
    let ext = path
        .extension()
        .map(OsStr::to_string_lossy)
        .as_deref()
        .map(str::to_lowercase);
    let file_type = match ext.as_deref() {
        Some("b3d") => FileType::B3D,
        Some("csv") => FileType::CSV,
        _ => return None, // TODO: Use result not option
    };

    let result = read_convert_utf8(path).await.ok()?; // TODO: Use result not option

    Some(generate_meshes(post_process(create_instructions(&result, file_type))))
}
