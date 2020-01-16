use crate::*;
use bve::parse::mesh;
use bve::{ColorU8RGB, ColorU8RGBA};
use std::ffi::CStr;
use std::ptr::null;

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

// Opaque
pub struct Texture_Set {
    pub inner: mesh::TextureSet,
}

impl From<mesh::TextureSet> for Texture_Set {
    #[inline]
    fn from(other: mesh::TextureSet) -> Self {
        Texture_Set { inner: other }
    }
}

impl Into<mesh::TextureSet> for Texture_Set {
    #[inline]
    fn into(self) -> mesh::TextureSet {
        self.inner
    }
}

#[must_use]
#[no_mangle]
pub unsafe extern "C" fn BVE_Texture_Set_len(ptr: *const Texture_Set) -> libc::size_t {
    (*ptr).inner.len()
}

#[no_mangle]
pub unsafe extern "C" fn BVE_Texture_Set_add(ptr: *mut Texture_Set, value: *const c_char) -> libc::size_t {
    (*ptr).inner.add(&CStr::from_ptr(value).to_string_lossy())
}

#[must_use]
#[no_mangle]
/// C Interface for [`bve::parse::mesh::TextureSet::lookup`]. Pointer returned points to an owned copy of the texture
/// name. Must be deleted by [`bve_native::bve_delete_string`] If the lookup fails, output is null.
pub unsafe extern "C" fn BVE_Texture_Set_lookup(ptr: *const Texture_Set, idx: libc::size_t) -> *const c_char {
    let result = (*ptr).inner.lookup(idx);
    match result {
        Some(s) => string_to_owned_ptr(s),
        None => null(),
    }
}

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
            emission_color: other.emission_color.into(),
        }
    }
}

impl Into<mesh::Texture> for Mesh_Texture {
    fn into(self) -> mesh::Texture {
        mesh::Texture {
            texture_id: self.texture_id.into(),
            decal_transparent_color: self.decal_transparent_color.into(),
            emission_color: self.emission_color.into(),
        }
    }
}

#[repr(C)]
pub struct Mesh {
    pub vertices: CVector<Vertex>,
    pub indices: CVector<libc::size_t>,
    pub texture: Mesh_Texture,
    pub color: ColorU8RGBA,
    pub blend_mode: mesh::BlendMode,
    pub glow: mesh::Glow,
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

#[repr(C, u8)]
pub enum Mesh_Error_Kind {
    UTF8 {
        column: COption<u64>,
    },
    OutOfBounds {
        idx: usize,
    },
    DeprecatedInstruction {
        /// Owning. Must be deleted by [`bve_native::bve_delete_string`]
        name: *const c_char,
    },
    UnknownInstruction {
        /// Owning. Must be deleted by [`bve_native::bve_delete_string`]
        name: *const c_char,
    },
    GenericCSV {
        /// Owning. Must be deleted by [`bve_native::bve_delete_string`]
        msg: *const c_char,
    },
    UnknownCSV,
}

impl From<mesh::MeshErrorKind> for Mesh_Error_Kind {
    fn from(other: mesh::MeshErrorKind) -> Self {
        match other {
            mesh::MeshErrorKind::UTF8 { column } => Mesh_Error_Kind::UTF8 { column: column.into() },
            mesh::MeshErrorKind::OutOfBounds { idx } => Mesh_Error_Kind::OutOfBounds { idx },
            mesh::MeshErrorKind::DeprecatedInstruction { name } => Mesh_Error_Kind::DeprecatedInstruction {
                name: string_to_owned_ptr(&name),
            },
            mesh::MeshErrorKind::UnknownInstruction { name } => Mesh_Error_Kind::UnknownInstruction {
                name: string_to_owned_ptr(&name),
            },
            mesh::MeshErrorKind::GenericCSV { msg } => Mesh_Error_Kind::GenericCSV {
                msg: string_to_owned_ptr(&msg),
            },
            mesh::MeshErrorKind::UnknownCSV => Mesh_Error_Kind::UnknownCSV,
        }
    }
}

impl Into<mesh::MeshErrorKind> for Mesh_Error_Kind {
    fn into(self) -> mesh::MeshErrorKind {
        match self {
            Mesh_Error_Kind::UTF8 { column } => mesh::MeshErrorKind::UTF8 { column: column.into() },
            Mesh_Error_Kind::OutOfBounds { idx } => mesh::MeshErrorKind::OutOfBounds { idx },
            Mesh_Error_Kind::DeprecatedInstruction { name } => mesh::MeshErrorKind::DeprecatedInstruction {
                name: unsafe { owned_ptr_to_string(name as *mut c_char) },
            },
            Mesh_Error_Kind::UnknownInstruction { name } => mesh::MeshErrorKind::UnknownInstruction {
                name: unsafe { owned_ptr_to_string(name as *mut c_char) },
            },
            Mesh_Error_Kind::GenericCSV { msg } => mesh::MeshErrorKind::GenericCSV {
                msg: unsafe { owned_ptr_to_string(msg as *mut c_char) },
            },
            Mesh_Error_Kind::UnknownCSV => mesh::MeshErrorKind::UnknownCSV,
        }
    }
}

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

#[no_mangle]
pub unsafe extern "C" fn bve_parse_mesh_from_string(
    string: *const c_char,
    file_type: mesh::FileType,
) -> Parsed_Static_Object {
    let result = mesh::mesh_from_str(&unowned_ptr_to_str(&string), file_type);
    result.into()
}
