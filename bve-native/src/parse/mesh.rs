//! C interface for [`bve::parse::mesh`] for parsing b3d/csv files.

use crate::interfaces::User_Error_Data;
use crate::parse::Span;
use crate::*;
use bve::parse::{mesh, UserError};
use bve_derive::c_interface;

pub use mesh::BlendMode;
pub use mesh::FileType;
pub use mesh::Glow;
pub use mesh::GlowAttenuationMode;

/// C safe wrapper for [`MeshError`](bve::parse::mesh::MeshError).
///
/// # Safety
///
/// - Must be destroyed as part of its parent [`load::mesh::Loaded_Static_Mesh`].
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Mesh_Error {
    pub location: Span,
    pub kind: Mesh_Error_Kind,
}

impl From<mesh::MeshError> for Mesh_Error {
    fn from(other: mesh::MeshError) -> Self {
        Self {
            location: other.location.into(),
            kind: other.kind.into(),
        }
    }
}

impl Into<mesh::MeshError> for Mesh_Error {
    fn into(self) -> mesh::MeshError {
        mesh::MeshError {
            location: self.location.into(),
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
/// - Must be destroyed as part of its parent [`load::mesh::Loaded_Static_Mesh`].
#[repr(C, u8)]
#[derive(Debug)]
pub enum Mesh_Error_Kind {
    UTF8 {
        column: COption<u64>,
    },
    OutOfBounds {
        idx: usize,
    },
    UnknownInstruction {
        name: *const c_char,
    },
    GenericCSV {
        msg: *const c_char,
        msg_english: *const c_char,
    },
    UnknownCSV,
}

impl Clone for Mesh_Error_Kind {
    fn clone(&self) -> Self {
        match self {
            Self::UTF8 { column } => Self::UTF8 { column: *column },
            Self::OutOfBounds { idx } => Self::OutOfBounds { idx: *idx },
            Self::UnknownInstruction { name } => unsafe {
                Self::UnknownInstruction {
                    name: copy_string(*name),
                }
            },
            Self::GenericCSV { msg, msg_english } => unsafe {
                Self::GenericCSV {
                    msg: copy_string(*msg),
                    msg_english: copy_string(*msg_english),
                }
            },
            Self::UnknownCSV => Self::UnknownCSV,
        }
    }
}

impl From<mesh::MeshErrorKind> for Mesh_Error_Kind {
    fn from(other: mesh::MeshErrorKind) -> Self {
        match other {
            mesh::MeshErrorKind::UTF8 { column } => Self::UTF8 { column: column.into() },
            mesh::MeshErrorKind::OutOfBounds { idx } => Self::OutOfBounds { idx },
            mesh::MeshErrorKind::UnknownInstruction { name } => Self::UnknownInstruction {
                name: str_to_owned_ptr(&name),
            },
            mesh::MeshErrorKind::GenericCSV { msg, msg_english } => Self::GenericCSV {
                msg: str_to_owned_ptr(&msg),
                msg_english: str_to_owned_ptr(&msg_english),
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
            Self::UnknownInstruction { name } => mesh::MeshErrorKind::UnknownInstruction {
                name: unsafe { owned_ptr_to_string(name as *mut c_char) },
            },
            Self::GenericCSV { msg, msg_english } => mesh::MeshErrorKind::GenericCSV {
                msg: unsafe { owned_ptr_to_string(msg as *mut c_char) },
                msg_english: unsafe { owned_ptr_to_string(msg_english as *mut c_char) },
            },
            Self::UnknownCSV => mesh::MeshErrorKind::UnknownCSV,
        }
    }
}

/// Get the localization and error data for a given error. C Interface for [`mesh::MeshError`]'s implementation of
/// [`bve::parse::UserError`].
///
/// # Safety
///
/// - `error` must be non-null, pointing to a valid Mesh_Error
#[c_interface]
pub unsafe extern "C" fn BVE_Mesh_Error_to_data(error: &Mesh_Error) -> User_Error_Data {
    let rust: mesh::MeshError = error.clone().into();
    rust.to_data().into()
}

/// C safe wrapper for [`MeshWarning`](bve::parse::mesh::MeshWarning).
///
/// # Safety
///
/// - Must be destroyed as part of its parent [`load::mesh::Loaded_Static_Mesh`].
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Mesh_Warning {
    pub location: Span,
    pub kind: Mesh_Warning_Kind,
}

impl From<mesh::MeshWarning> for Mesh_Warning {
    fn from(other: mesh::MeshWarning) -> Self {
        Self {
            location: other.location.into(),
            kind: other.kind.into(),
        }
    }
}

impl Into<mesh::MeshWarning> for Mesh_Warning {
    fn into(self) -> mesh::MeshWarning {
        mesh::MeshWarning {
            location: self.location.into(),
            kind: self.kind.into(),
        }
    }
}

/// C safe wrapper for [`MeshWarningKind`](bve::parse::mesh::MeshWarningKind).
///
/// # Safety
///
/// - Only read the union value that the `tag`/`determinant` says is inside the enum.
/// - Reading another value results in UB.
/// - Must be destroyed as part of its parent [`load::mesh::Loaded_Static_Mesh`].
#[repr(C, u8)]
#[derive(Debug)]
pub enum Mesh_Warning_Kind {
    UselessInstruction { name: *const c_char },
}

impl Clone for Mesh_Warning_Kind {
    fn clone(&self) -> Self {
        match self {
            Self::UselessInstruction { name } => unsafe {
                Self::UselessInstruction {
                    name: copy_string(*name),
                }
            },
        }
    }
}

impl From<mesh::MeshWarningKind> for Mesh_Warning_Kind {
    fn from(other: mesh::MeshWarningKind) -> Self {
        match other {
            mesh::MeshWarningKind::UselessInstruction { name } => Self::UselessInstruction {
                name: str_to_owned_ptr(&name),
            },
        }
    }
}

impl Into<mesh::MeshWarningKind> for Mesh_Warning_Kind {
    fn into(self) -> mesh::MeshWarningKind {
        match self {
            Self::UselessInstruction { name } => mesh::MeshWarningKind::UselessInstruction {
                name: unsafe { owned_ptr_to_string(name as *mut c_char) },
            },
        }
    }
}

/// Get the localization and error data for a given warnings. C Interface for [`mesh::MeshWarning`]'s implementation of
/// [`bve::parse::UserError`].
///
/// # Safety
///
/// - `warning` must be non-null, pointing to a valid Mesh_Warning
#[c_interface]
pub unsafe extern "C" fn BVE_Mesh_Warnings_to_data(warning: &Mesh_Warning) -> User_Error_Data {
    let rust: mesh::MeshWarning = warning.clone().into();
    rust.to_data().into()
}
