use crate::{
    parse::mesh::{BlendMode, Glow, Mesh_Error, Mesh_Warning},
    str_to_owned_ptr, unowned_ptr_to_str, COption, CVector,
};
use async_std::task::block_on;
use bve::{load::mesh, ColorU8RGB, ColorU8RGBA};
use bve_derive::c_interface;
use libc::c_char;
use std::{
    ffi::CStr,
    ptr::{null, null_mut},
};

pub use mesh::Vertex;

/// C safe wrapper for [`LoadedStaticMesh`](bve::load::mesh::LoadedStaticMesh).
///
/// # Safety
///
/// - It and all child objects must be deleted by calling [`bve_delete_loaded_static_mesh`].
#[repr(C)]
pub struct Loaded_Static_Mesh {
    pub meshes: CVector<Mesh>,
    pub textures: *mut Texture_Set,
    pub warnings: CVector<Mesh_Warning>,
    pub errors: CVector<Mesh_Error>,
}

impl From<mesh::LoadedStaticMesh> for Loaded_Static_Mesh {
    #[inline]
    fn from(other: mesh::LoadedStaticMesh) -> Self {
        Self {
            meshes: other.meshes.into(),
            textures: Box::into_raw(Box::new(other.textures.into())),
            warnings: other.warnings.into(),
            errors: other.errors.into(),
        }
    }
}

impl Into<mesh::LoadedStaticMesh> for Loaded_Static_Mesh {
    #[inline]
    fn into(self) -> mesh::LoadedStaticMesh {
        mesh::LoadedStaticMesh {
            meshes: self.meshes.into(),
            textures: unsafe { *Box::from_raw(self.textures) }.into(),
            warnings: self.warnings.into(),
            errors: self.errors.into(),
        }
    }
}

/// C Destructor for [`Loaded_Static_Mesh`].
///
/// # Safety
///
/// - Object provided must be able to be reassembled into a rust datastructure before being deleted. This means the
///   invariants of all of rust's equivalent datastructure must be upheld.
#[c_interface]
pub unsafe extern "C" fn bve_delete_loaded_static_mesh(object: *mut Loaded_Static_Mesh) {
    if object.is_null() {
        let _reassembled: mesh::LoadedStaticMesh = (*Box::from_raw(object)).into();
        // Object safely deleted
    }
}

/// C safe wrapper for [`TextureSet`](bve::load::mesh::TextureSet).
///
/// Opaque structure which wraps a set of texture names.
///
/// # Members
///
/// Accessible through the "member" functions:
/// - [`BVE_Texture_Set_len`] for [`TextureSet::len`](bve::load::mesh::TextureSet::len)
/// - [`BVE_Texture_Set_add`] for [`TextureSet::add`](bve::load::mesh::TextureSet::add)
/// - [`BVE_Texture_Set_lookup`] for [`TextureSet::lookup`](bve::load::mesh::TextureSet::lookup)
///
/// # Safety
///
/// - Must be destroyed as part of its parent [`Loaded_Static_Mesh`].
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
/// C "member function" for [`TextureSet::len`](bve::load::mesh::TextureSet::len).
///
/// # Safety
///
/// - `ptr` must be non-null.
pub unsafe extern "C" fn BVE_Texture_Set_len(ptr: *const Texture_Set) -> libc::size_t {
    (*ptr).inner.len()
}

#[c_interface]
/// C "member function" for [`TextureSet::add`](bve::load::mesh::TextureSet::add).
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
/// C "member function" for [`TextureSet::lookup`](bve::load::mesh::TextureSet::lookup).
///
/// # Safety
///
/// - Pointer returned points to an owned **copy** of the texture name.
/// - Returned pointer must be deleted by [`crate::bve_delete_string`].
/// - If the lookup fails, output is null.
pub unsafe extern "C" fn BVE_Texture_Set_lookup(ptr: *const Texture_Set, idx: libc::size_t) -> *c_char {
    let result = (*ptr).inner.lookup(idx);
    match result {
        Some(s) => str_to_owned_ptr(s),
        None => null(),
    }
}

/// C safe wrapper for [`Texture`](bve::load::mesh::Texture).
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

/// C safe wrapper for [`Mesh`](bve::load::mesh::Mesh).
///
/// # Safety
///
/// - Must be destroyed as part of its parent [`Loaded_Static_Mesh`].
#[repr(C)]
pub struct Mesh {
    pub vertices: CVector<mesh::Vertex>,
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

/// C Interface for [`load_mesh_from_file`](bve::load::mesh::load_mesh_from_file).
///
/// # Safety
///
/// - `file` must be non-null and null terminated.
/// - Result must be properly deleted.
#[must_use]
#[c_interface]
pub unsafe extern "C" fn bve_load_mesh_from_file(file: *const c_char) -> *mut Loaded_Static_Mesh {
    let result = block_on(mesh::load_mesh_from_file(unowned_ptr_to_str(&file).as_ref()));
    match result {
        Some(m) => Box::into_raw(Box::new(m.into())),
        None => null_mut(),
    }
}
