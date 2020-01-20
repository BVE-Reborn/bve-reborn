//! C interface for [`bve::parse::mesh`] for parsing b3d/csv files.
//!
//! Currently only a single entry point is exposed: [`bve_parse_mesh_from_string`].

use crate::*;
use bve::parse::mesh;
use bve::{ColorU8RGB, ColorU8RGBA};
use bve_derive::c_interface;
use std::ffi::CStr;
use std::ptr::null;

pub use mesh::BlendMode;
pub use mesh::FileType;
pub use mesh::Glow;
pub use mesh::GlowAttenuationMode;
pub use mesh::Vertex;

/// C safe wrapper for [`ParsedStaticObject`](bve::parse::mesh::ParsedStaticObject).
///
/// # Safety
///
/// - It and all child objects must be deleted by calling [`bve_delete_parsed_static_object`].
#[repr(C)]
pub struct Parsed_Static_Object {
    pub meshes: CVector<Mesh>,
    pub textures: *mut Texture_Set,
    pub errors: CVector<Mesh_Error>,
}

impl From<mesh::ParsedStaticObject> for Parsed_Static_Object {
    #[inline]
    fn from(other: mesh::ParsedStaticObject) -> Self {
        Self {
            meshes: other.meshes.into(),
            textures: Box::into_raw(Box::new(other.textures.into())),
            errors: other.errors.into(),
        }
    }
}

impl Into<mesh::ParsedStaticObject> for Parsed_Static_Object {
    #[inline]
    fn into(self) -> mesh::ParsedStaticObject {
        mesh::ParsedStaticObject {
            meshes: self.meshes.into(),
            textures: unsafe { *Box::from_raw(self.textures) }.into(),
            errors: self.errors.into(),
        }
    }
}

/// C Destructor for [`Parsed_Static_Object`].
///
/// # Safety
///
/// - Object provided must be able to be reassembled into a rust datastructure before being deleted. This means the
///   invariants of all of rust's equivalent datastructure must be upheld.
#[c_interface]
pub unsafe extern "C" fn bve_delete_parsed_static_object(object: Parsed_Static_Object) {
    let _reassembled: mesh::ParsedStaticObject = object.into();
    // Object safely deleted
}

/// C safe wrapper for [`TextureSet`](bve::parse::mesh::TextureSet).
///
/// Opaque structure which wraps a set of texture names.
///
/// # Members
///
/// Accessible through the "member" functions:
/// - [`BVE_Texture_Set_len`] for [`TextureSet::len`](bve::parse::mesh::TextureSet::len)
/// - [`BVE_Texture_Set_add`] for [`TextureSet::add`](bve::parse::mesh::TextureSet::add)
/// - [`BVE_Texture_Set_lookup`] for [`TextureSet::lookup`](bve::parse::mesh::TextureSet::lookup)
///
/// # Safety
///
/// - Must be destroyed as part of its parent [`Parsed_Static_Object`].
pub struct Texture_Set {
    pub inner: mesh::TextureSet,
}

impl From<mesh::TextureSet> for Texture_Set {
    #[inline]
    fn from(other: mesh::TextureSet) -> Self {
        Self { inner: other }
    }
}

impl Into<mesh::TextureSet> for Texture_Set {
    #[inline]
    fn into(self) -> mesh::TextureSet {
        self.inner
    }
}

#[must_use]
#[c_interface]
/// C "member function" for [`TextureSet::len`](bve::parse::mesh::TextureSet::len).
///
/// # Safety
///
/// - `ptr` must be non-null.
pub unsafe extern "C" fn BVE_Texture_Set_len(ptr: *const Texture_Set) -> libc::size_t {
    (*ptr).inner.len()
}

#[c_interface]
/// C "member function" for [`TextureSet::add`](bve::parse::mesh::TextureSet::add).
///
/// # Safety
///
/// - `ptr` must be non-null.
/// - `value` Must be a valid null-terminated string. Non-utf8 is permitted, though escaped.
pub unsafe extern "C" fn BVE_Texture_Set_add(ptr: *mut Texture_Set, value: *const c_char) -> libc::size_t {
    (*ptr).inner.add(&CStr::from_ptr(value).to_string_lossy())
}

#[must_use]
#[c_interface]
/// C "member function" for [`TextureSet::lookup`](bve::parse::mesh::TextureSet::lookup).
///
/// # Safety
///
/// - Pointer returned points to an owned **copy** of the texture name.
/// - Returned pointer must be deleted by [`bve_delete_string`].
/// - If the lookup fails, output is null.
pub unsafe extern "C" fn BVE_Texture_Set_lookup(ptr: *const Texture_Set, idx: libc::size_t) -> *const c_char {
    let result = (*ptr).inner.lookup(idx);
    match result {
        Some(s) => string_to_owned_ptr(s),
        None => null(),
    }
}

/// C safe wrapper for [`Texture`](bve::parse::mesh::Texture).
#[repr(C)]
pub struct Mesh_Texture {
    pub texture_id: COption<usize>,
    pub decal_transparent_color: COption<ColorU8RGB>,
    pub emission_color: ColorU8RGB,
}

impl From<mesh::Texture> for Mesh_Texture {
    fn from(other: mesh::Texture) -> Self {
        Self {
            texture_id: other.texture_id.into(),
            decal_transparent_color: other.decal_transparent_color.into(),
            emission_color: other.emission_color,
        }
    }
}

impl Into<mesh::Texture> for Mesh_Texture {
    fn into(self) -> mesh::Texture {
        mesh::Texture {
            texture_id: self.texture_id.into(),
            decal_transparent_color: self.decal_transparent_color.into(),
            emission_color: self.emission_color,
        }
    }
}

/// C safe wrapper for [`Mesh`](bve::parse::mesh::Mesh).
///
/// # Safety
///
/// - Must be destroyed as part of its parent [`Parsed_Static_Object`].
#[repr(C)]
pub struct Mesh {
    pub vertices: CVector<Vertex>,
    pub indices: CVector<libc::size_t>,
    pub texture: Mesh_Texture,
    pub color: ColorU8RGBA,
    pub blend_mode: BlendMode,
    pub glow: Glow,
}

impl From<mesh::Mesh> for Mesh {
    fn from(other: mesh::Mesh) -> Self {
        Self {
            vertices: other.vertices.into(),
            indices: other.indices.into(),
            texture: other.texture.into(),
            color: other.color,
            blend_mode: other.blend_mode,
            glow: other.glow,
        }
    }
}

impl Into<mesh::Mesh> for Mesh {
    fn into(self) -> mesh::Mesh {
        mesh::Mesh {
            vertices: self.vertices.into(),
            indices: self.indices.into(),
            texture: self.texture.into(),
            color: self.color,
            blend_mode: self.blend_mode,
            glow: self.glow,
        }
    }
}

/// C safe wrapper for [`MeshError`](bve::parse::mesh::MeshError).
///
/// # Safety
///
/// - Must be destroyed as part of its parent [`Parsed_Static_Object`].
#[repr(C)]
pub struct Mesh_Error {
    pub span: Span,
    pub kind: Mesh_Error_Kind,
}

impl From<mesh::MeshError> for Mesh_Error {
    fn from(other: mesh::MeshError) -> Self {
        Self {
            span: other.span.into(),
            kind: other.kind.into(),
        }
    }
}

impl Into<mesh::MeshError> for Mesh_Error {
    fn into(self) -> mesh::MeshError {
        mesh::MeshError {
            span: self.span.into(),
            kind: self.kind.into(),
        }
    }
}

/// C safe wrapper for [`MeshErrorKind`](bve::parse::mesh::MeshErrorKind).
///
/// # Safety
///
/// - Only read the union value that the `tag`/`determinant` says is inside the enum.
/// - Reading another value results in UB.
/// - Must be destroyed as part of its parent [`Parsed_Static_Object`].
#[repr(C, u8)]
pub enum Mesh_Error_Kind {
    UTF8 { column: COption<u64> },
    OutOfBounds { idx: usize },
    DeprecatedInstruction { name: *const c_char },
    UnknownInstruction { name: *const c_char },
    GenericCSV { msg: *const c_char },
    UnknownCSV,
}

impl From<mesh::MeshErrorKind> for Mesh_Error_Kind {
    fn from(other: mesh::MeshErrorKind) -> Self {
        match other {
            mesh::MeshErrorKind::UTF8 { column } => Self::UTF8 { column: column.into() },
            mesh::MeshErrorKind::OutOfBounds { idx } => Self::OutOfBounds { idx },
            mesh::MeshErrorKind::DeprecatedInstruction { name } => Self::DeprecatedInstruction {
                name: string_to_owned_ptr(&name),
            },
            mesh::MeshErrorKind::UnknownInstruction { name } => Self::UnknownInstruction {
                name: string_to_owned_ptr(&name),
            },
            mesh::MeshErrorKind::GenericCSV { msg } => Self::GenericCSV {
                msg: string_to_owned_ptr(&msg),
            },
            mesh::MeshErrorKind::UnknownCSV => Self::UnknownCSV,
        }
    }
}

impl Into<mesh::MeshErrorKind> for Mesh_Error_Kind {
    fn into(self) -> mesh::MeshErrorKind {
        match self {
            Self::UTF8 { column } => mesh::MeshErrorKind::UTF8 { column: column.into() },
            Self::OutOfBounds { idx } => mesh::MeshErrorKind::OutOfBounds { idx },
            Self::DeprecatedInstruction { name } => mesh::MeshErrorKind::DeprecatedInstruction {
                name: unsafe { owned_ptr_to_string(name as *mut c_char) },
            },
            Self::UnknownInstruction { name } => mesh::MeshErrorKind::UnknownInstruction {
                name: unsafe { owned_ptr_to_string(name as *mut c_char) },
            },
            Self::GenericCSV { msg } => mesh::MeshErrorKind::GenericCSV {
                msg: unsafe { owned_ptr_to_string(msg as *mut c_char) },
            },
            Self::UnknownCSV => mesh::MeshErrorKind::UnknownCSV,
        }
    }
}

/// C safe wrapper for [`Span`](bve::parse::mesh::Span).
#[repr(C)]
pub struct Span {
    pub line: COption<u64>,
}

impl From<mesh::Span> for Span {
    fn from(other: mesh::Span) -> Self {
        Self {
            line: other.line.into(),
        }
    }
}

impl Into<mesh::Span> for Span {
    fn into(self) -> mesh::Span {
        mesh::Span { line: self.line.into() }
    }
}

/// C Interface for [`mesh_from_str`](bve::parse::mesh::mesh_from_str).
///
/// # Safety
///
/// - `string` must be non-null and null terminated. May be invalid utf8.
/// - `file_type` must be a valid enumeration.
/// - Result must be properly deleted.
#[must_use]
#[c_interface]
pub unsafe extern "C" fn bve_parse_mesh_from_string(
    string: *const c_char,
    file_type: FileType,
) -> Parsed_Static_Object {
    let result = mesh::mesh_from_str(&unowned_ptr_to_str(&string), file_type);
    result.into()
}
