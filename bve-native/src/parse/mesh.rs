use bve::parse::mesh::{mesh_from_str, BlendMode, FileType, Glow, TextureSet, Vertex};
use bve::{ColorU8RGB, ColorU8RGBA};
use std::ffi::{CStr, CString};
use std::os::raw::*;
use std::ptr::null;

bve_vector!(BVE_Vector_Mesh, BVE_Mesh);
bve_vector!(BVE_Vector_Mesh_Error, BVE_Mesh_Error);

#[repr(C)]
pub struct BVE_Parsed_Static_Object {
    pub meshes: BVE_Vector_Mesh,
    pub textures: *mut BVE_Texture_Set,
    pub errors: BVE_Vector_Mesh_Error,
}

// Opaque
pub struct BVE_Texture_Set {
    pub inner: TextureSet,
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
/// C Interface for [`bve::parse::mesh::TextureSet::lookup`]. Pointer returned points into the
/// texture set and is only valid for as long as it is. It may not be modified. If the lookup fails,
/// output is null.
pub unsafe extern "C" fn BVE_Texture_Set_lookup(ptr: *const BVE_Texture_Set, idx: libc::size_t) -> *const c_char {
    let result = (*ptr).inner.lookup(idx);
    match result {
        Some(s) => CString::new(s).map(|c| c.as_ptr()).unwrap_or(null()),
        None => null(),
    }
}

bve_option!(BVE_Option_size_t, libc::size_t);
bve_option!(BVE_Option_ColorU8RGB, ColorU8RGB);

#[repr(C)]
pub struct BVE_Mesh_Texture {
    pub texture_id: BVE_Option_size_t,
    pub decal_transparent_color: BVE_Option_ColorU8RGB,
    pub emission_color: ColorU8RGB,
}

bve_vector!(BVE_Vector_Vertex, Vertex);
bve_vector!(BVE_Vector_size_t, libc::size_t);

#[repr(C)]
pub struct BVE_Mesh {
    pub vertices: BVE_Vector_Vertex,
    pub indices: BVE_Vector_size_t,
    pub texture: BVE_Mesh_Texture,
    pub color: ColorU8RGBA,
    pub blend_mode: BlendMode,
    pub glow: Glow,
}

#[repr(C)]
pub struct BVE_Mesh_Error {
    pub span: BVE_Span,
    pub kind: BVE_Mesh_Error_Kind,
}

bve_option!(BVE_Option_unsigned_long_long, c_ulonglong);

#[repr(C, u8)]
pub enum BVE_Mesh_Error_Kind {
    UTF8 {
        column: BVE_Option_unsigned_long_long,
    },
    OutOfBounds {
        idx: usize,
    },
    DeprecatedInstruction {
        /// Owning
        name: *const c_char,
    },
    UnknownInstruction {
        /// Owning
        name: *const c_char,
    },
    GenericCSV {
        /// Owning
        name: *const c_char,
    },
    UnknownCSV,
}

#[repr(C)]
pub struct BVE_Span {
    pub line: BVE_Option_unsigned_long_long,
}

#[no_mangle]
pub unsafe extern "C" fn bve_parse_mesh_from_string(
    string: *const c_char,
    file_type: FileType,
) -> BVE_Parsed_Static_Object {
    let result = mesh_from_str(&CStr::from_ptr(string).to_string_lossy(), file_type);
    unimplemented!();
}
