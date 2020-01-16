use crate::*;
use bve::parse::mesh::*;
use bve::{ColorU8RGB, ColorU8RGBA};
use std::ffi::CStr;
use std::ptr::null;

#[repr(C)]
pub struct BVE_Parsed_Static_Object {
    pub meshes: BVE_Vector_Mesh,
    pub textures: *mut BVE_Texture_Set,
    pub errors: BVE_Vector_Mesh_Error,
}

impl From<ParsedStaticObject> for BVE_Parsed_Static_Object {
    #[inline]
    fn from(other: ParsedStaticObject) -> Self {
        Self {
            meshes: other.meshes.into(),
            textures: Box::into_raw(Box::new(other.textures.into())),
            errors: other.errors.into(),
        }
    }
}

impl Into<ParsedStaticObject> for BVE_Parsed_Static_Object {
    #[inline]
    fn into(self) -> ParsedStaticObject {
        ParsedStaticObject {
            meshes: self.meshes.into(),
            textures: unsafe { *Box::from_raw(self.textures) }.into(),
            errors: self.errors.into(),
        }
    }
}

// Opaque
pub struct BVE_Texture_Set {
    pub inner: TextureSet,
}

impl From<TextureSet> for BVE_Texture_Set {
    #[inline]
    fn from(other: TextureSet) -> Self {
        BVE_Texture_Set { inner: other }
    }
}

impl Into<TextureSet> for BVE_Texture_Set {
    #[inline]
    fn into(self) -> TextureSet {
        self.inner
    }
}

#[must_use]
#[no_mangle]
pub unsafe extern "C" fn BVE_Texture_Set_len(ptr: *const BVE_Texture_Set) -> libc::size_t {
    (*ptr).inner.len()
}

#[no_mangle]
pub unsafe extern "C" fn BVE_Texture_Set_add(ptr: *mut BVE_Texture_Set, value: *const c_char) -> libc::size_t {
    (*ptr).inner.add(&CStr::from_ptr(value).to_string_lossy())
}

#[must_use]
#[no_mangle]
/// C Interface for [`bve::parse::mesh::TextureSet::lookup`]. Pointer returned points to an owned copy of the texture
/// name. Must be deleted by [`bve_native::bve_delete_string`] If the lookup fails, output is null.
pub unsafe extern "C" fn BVE_Texture_Set_lookup(ptr: *const BVE_Texture_Set, idx: libc::size_t) -> *const c_char {
    let result = (*ptr).inner.lookup(idx);
    match result {
        Some(s) => string_to_owned_ptr(s),
        None => null(),
    }
}

#[repr(C)]
pub struct BVE_Mesh_Texture {
    pub texture_id: BVE_Option_size_t,
    pub decal_transparent_color: BVE_Option_ColorU8RGB,
    pub emission_color: ColorU8RGB,
}

impl From<Texture> for BVE_Mesh_Texture {
    fn from(other: Texture) -> Self {
        Self {
            texture_id: other.texture_id.into(),
            decal_transparent_color: other.decal_transparent_color.into(),
            emission_color: other.emission_color.into(),
        }
    }
}

impl Into<Texture> for BVE_Mesh_Texture {
    fn into(self) -> Texture {
        Texture {
            texture_id: self.texture_id.into(),
            decal_transparent_color: self.decal_transparent_color.into(),
            emission_color: self.emission_color.into(),
        }
    }
}

#[repr(C)]
pub struct BVE_Mesh {
    pub vertices: BVE_Vector_Vertex,
    pub indices: BVE_Vector_size_t,
    pub texture: BVE_Mesh_Texture,
    pub color: ColorU8RGBA,
    pub blend_mode: BlendMode,
    pub glow: Glow,
}

impl From<Mesh> for BVE_Mesh {
    fn from(other: Mesh) -> Self {
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

impl Into<Mesh> for BVE_Mesh {
    fn into(self) -> Mesh {
        Mesh {
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
pub struct BVE_Mesh_Error {
    pub span: BVE_Span,
    pub kind: BVE_Mesh_Error_Kind,
}

impl From<MeshError> for BVE_Mesh_Error {
    fn from(other: MeshError) -> Self {
        Self {
            span: other.span.into(),
            kind: other.kind.into(),
        }
    }
}

impl Into<MeshError> for BVE_Mesh_Error {
    fn into(self) -> MeshError {
        MeshError {
            span: self.span.into(),
            kind: self.kind.into(),
        }
    }
}

#[repr(C, u8)]
pub enum BVE_Mesh_Error_Kind {
    UTF8 {
        column: BVE_Option_unsigned_long_long,
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

impl From<MeshErrorKind> for BVE_Mesh_Error_Kind {
    fn from(other: MeshErrorKind) -> Self {
        match other {
            MeshErrorKind::UTF8 { column } => BVE_Mesh_Error_Kind::UTF8 { column: column.into() },
            MeshErrorKind::OutOfBounds { idx } => BVE_Mesh_Error_Kind::OutOfBounds { idx },
            MeshErrorKind::DeprecatedInstruction { name } => BVE_Mesh_Error_Kind::DeprecatedInstruction {
                name: string_to_owned_ptr(&name),
            },
            MeshErrorKind::UnknownInstruction { name } => BVE_Mesh_Error_Kind::UnknownInstruction {
                name: string_to_owned_ptr(&name),
            },
            MeshErrorKind::GenericCSV { msg } => BVE_Mesh_Error_Kind::GenericCSV {
                msg: string_to_owned_ptr(&msg),
            },
            MeshErrorKind::UnknownCSV => BVE_Mesh_Error_Kind::UnknownCSV,
        }
    }
}

impl Into<MeshErrorKind> for BVE_Mesh_Error_Kind {
    fn into(self) -> MeshErrorKind {
        match self {
            BVE_Mesh_Error_Kind::UTF8 { column } => MeshErrorKind::UTF8 { column: column.into() },
            BVE_Mesh_Error_Kind::OutOfBounds { idx } => MeshErrorKind::OutOfBounds { idx },
            BVE_Mesh_Error_Kind::DeprecatedInstruction { name } => MeshErrorKind::DeprecatedInstruction {
                name: unsafe { owned_ptr_to_string(name as *mut c_char) },
            },
            BVE_Mesh_Error_Kind::UnknownInstruction { name } => MeshErrorKind::UnknownInstruction {
                name: unsafe { owned_ptr_to_string(name as *mut c_char) },
            },
            BVE_Mesh_Error_Kind::GenericCSV { msg } => MeshErrorKind::GenericCSV {
                msg: unsafe { owned_ptr_to_string(msg as *mut c_char) },
            },
            BVE_Mesh_Error_Kind::UnknownCSV => MeshErrorKind::UnknownCSV,
        }
    }
}

#[repr(C)]
pub struct BVE_Span {
    pub line: BVE_Option_unsigned_long_long,
}

impl From<Span> for BVE_Span {
    fn from(other: Span) -> Self {
        Self {
            line: other.line.into(),
        }
    }
}

impl Into<Span> for BVE_Span {
    fn into(self) -> Span {
        Span { line: self.line.into() }
    }
}

#[no_mangle]
pub unsafe extern "C" fn bve_parse_mesh_from_string(
    string: *const c_char,
    file_type: FileType,
) -> BVE_Parsed_Static_Object {
    let result = mesh_from_str(&unowned_ptr_to_str(string), file_type);
    result.into()
}
