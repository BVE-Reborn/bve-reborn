//! C interface for [`bve::parse::mesh`] for parsing b3d/csv files.

use crate::parse::Span;
use crate::*;
use bve::parse::mesh;

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

impl From<mesh::MeshErrorKind> for Mesh_Error_Kind {
    fn from(other: mesh::MeshErrorKind) -> Self {
        match other {
            mesh::MeshErrorKind::UTF8 { column } => Self::UTF8 { column: column.into() },
            mesh::MeshErrorKind::OutOfBounds { idx } => Self::OutOfBounds { idx },
            mesh::MeshErrorKind::UnknownInstruction { name } => Self::UnknownInstruction {
                name: string_to_owned_ptr(&name),
            },
            mesh::MeshErrorKind::GenericCSV { msg, msg_english } => Self::GenericCSV {
                msg: string_to_owned_ptr(&msg),
                msg_english: string_to_owned_ptr(&msg_english),
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

/// C safe wrapper for [`MeshWarning`](bve::parse::mesh::MeshWarning).
///
/// # Safety
///
/// - Must be destroyed as part of its parent [`load::mesh::Loaded_Static_Mesh`].
#[repr(C)]
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
pub enum Mesh_Warning_Kind {
    UselessInstruction { name: *const c_char },
}

impl From<mesh::MeshWarningKind> for Mesh_Warning_Kind {
    fn from(other: mesh::MeshWarningKind) -> Self {
        match other {
            mesh::MeshWarningKind::UselessInstruction { name } => Self::UselessInstruction {
                name: string_to_owned_ptr(&name),
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
