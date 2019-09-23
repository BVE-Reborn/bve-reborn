use crate::{ColorU8RGB, ColorU8RGBA};
use cgmath::{Vector2, Vector3};
use indexmap::IndexSet;

pub mod instructions;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    B3D,
    CSV,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshError {
    pub kind: MeshErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MeshErrorKind {
    UTF8Error { column: Option<u64> },
    GenericCSVError { msg: String },
    UnknownCSVError,
}

impl From<csv::Error> for MeshError {
    fn from(e: csv::Error) -> Self {
        match e.kind() {
            csv::ErrorKind::Deserialize {
                err: deserialize_error, ..
            } => match deserialize_error.kind() {
                csv::DeserializeErrorKind::InvalidUtf8(_) => MeshError {
                    kind: MeshErrorKind::UTF8Error {
                        column: deserialize_error.field().map(|f| f + 1),
                    },
                    span: e.position().into(),
                },
                csv::DeserializeErrorKind::Message(msg) => MeshError {
                    kind: MeshErrorKind::GenericCSVError { msg: msg.clone() },
                    span: e.position().into(),
                },
                csv::DeserializeErrorKind::Unsupported(msg) => MeshError {
                    kind: MeshErrorKind::GenericCSVError { msg: msg.clone() },
                    span: e.position().into(),
                },
                csv::DeserializeErrorKind::UnexpectedEndOfRow => MeshError {
                    kind: MeshErrorKind::GenericCSVError {
                        msg: "Not enough arguments".into(),
                    },
                    span: e.position().into(),
                },
                csv::DeserializeErrorKind::ParseFloat(ferr) => {
                    let message = format!(
                        "Float parsing error \"{}\" in csv column {}",
                        ferr,
                        deserialize_error
                            .field()
                            .map(|f| (f + 1).to_string())
                            .unwrap_or_else(|| "?".into())
                    );

                    MeshError {
                        kind: MeshErrorKind::GenericCSVError { msg: message },
                        span: e.position().into(),
                    }
                }
                csv::DeserializeErrorKind::ParseInt(ierr) => {
                    let message = format!(
                        "Int parsing error \"{}\" in csv column {}",
                        ierr,
                        deserialize_error
                            .field()
                            .map(|f| (f + 1).to_string())
                            .unwrap_or_else(|| "?".into())
                    );

                    MeshError {
                        kind: MeshErrorKind::GenericCSVError { msg: message },
                        span: e.position().into(),
                    }
                }
                csv::DeserializeErrorKind::ParseBool(berr) => {
                    let message = format!(
                        "Bool parsing error \"{}\" in csv column {}",
                        berr,
                        deserialize_error
                            .field()
                            .map(|f| (f + 1).to_string())
                            .unwrap_or_else(|| "?".into())
                    );

                    MeshError {
                        kind: MeshErrorKind::GenericCSVError { msg: message },
                        span: e.position().into(),
                    }
                }
            },
            csv::ErrorKind::Utf8 { err, .. } => MeshError {
                kind: MeshErrorKind::UTF8Error {
                    column: Some(err.field() as u64 + 1),
                },
                span: e.position().into(),
            },
            _ => MeshError {
                kind: MeshErrorKind::UnknownCSVError,
                span: e.position().into(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: Option<u64>,
}

impl<'a> From<Option<&'a csv::Position>> for Span {
    fn from(p: Option<&'a csv::Position>) -> Self {
        Self {
            line: p.map(|v| v.line()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStaticObject {
    pub meshes: Vec<Mesh>,
    pub textures: TextureFileSet,
    pub errors: Vec<MeshError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureFileSet {
    filenames: IndexSet<String>,
}

impl TextureFileSet {
    pub fn new() -> Self {
        TextureFileSet {
            filenames: IndexSet::new(),
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        TextureFileSet {
            filenames: IndexSet::with_capacity(size),
        }
    }

    pub fn add(&mut self, value: String) -> usize {
        self.filenames.insert_full(value).0
    }

    pub fn lookup(&self, idx: usize) -> Option<&str> {
        self.filenames.get_index(idx).map(|s| s.as_str())
    }

    pub fn merge(&mut self, other: TextureFileSet) {
        self.filenames.extend(other.filenames)
    }
}

impl Default for TextureFileSet {
    fn default() -> Self {
        TextureFileSet::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    pub texture_file: usize,
    pub decal_transparent_color: Option<ColorU8RGB>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u64>,
    pub face_data: Vec<FaceData>,
    pub texture: Texture,
    pub color: ColorU8RGBA,
    pub blend_mode: BlendMode,
    pub glow: Glow,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub coord: Vector2<f32>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceData {
    pub emission_color: ColorU8RGB,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Glow {
    pub attenuation_mode: GlowAttenuationMode,
    pub half_distance: u16,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum BlendMode {
    Normal,
    Additive,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum GlowAttenuationMode {
    DivideExponent2,
    DivideExponent4,
}
