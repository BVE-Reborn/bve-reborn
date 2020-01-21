use crate::parse::Span;

/// A single error in the parsing or evaluation of a mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshError {
    /// Info about the exact error.
    pub kind: MeshErrorKind,
    /// Location of the error within the file.
    pub span: Span,
}

/// Enum representing various types of errors encountered when parsing meshes
#[derive(Debug, Clone, PartialEq)]
pub enum MeshErrorKind {
    /// Invalid UTF-8. May be Shift-JIS or other encoding.
    UTF8 {
        /// Column of the error. Only Optional due to the CSV library.
        column: Option<u64>,
    },
    /// Index provided to vertex-specific command is out of bounds.
    OutOfBounds { idx: usize },
    /// Instruction no longer does anything anymore
    UselessInstruction { name: String },
    /// Unrecognized instruction
    UnknownInstruction {
        /// Instruction that is not recognized
        name: String,
    },
    /// Handled CSV error
    GenericCSV {
        /// Message provided by CSV Library
        msg: String,
    },
    /// Unknown csv error
    UnknownCSV,
}

impl From<csv::Error> for MeshError {
    #[must_use]
    fn from(e: csv::Error) -> Self {
        match e.kind() {
            csv::ErrorKind::Deserialize {
                err: deserialize_error, ..
            } => match deserialize_error.kind() {
                csv::DeserializeErrorKind::InvalidUtf8(_) => Self {
                    kind: MeshErrorKind::UTF8 {
                        column: deserialize_error.field().map(|f| f + 1),
                    },
                    span: e.position().into(),
                },
                csv::DeserializeErrorKind::Message(msg) | csv::DeserializeErrorKind::Unsupported(msg) => Self {
                    kind: MeshErrorKind::GenericCSV { msg: msg.clone() },
                    span: e.position().into(),
                },
                csv::DeserializeErrorKind::UnexpectedEndOfRow => Self {
                    kind: MeshErrorKind::GenericCSV {
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
                            .map_or_else(|| "?".into(), |f| (f + 1).to_string())
                    );

                    Self {
                        kind: MeshErrorKind::GenericCSV { msg: message },
                        span: e.position().into(),
                    }
                }
                csv::DeserializeErrorKind::ParseInt(ierr) => {
                    let message = format!(
                        "Int parsing error \"{}\" in csv column {}",
                        ierr,
                        deserialize_error
                            .field()
                            .map_or_else(|| "?".into(), |f| (f + 1).to_string())
                    );

                    Self {
                        kind: MeshErrorKind::GenericCSV { msg: message },
                        span: e.position().into(),
                    }
                }
                csv::DeserializeErrorKind::ParseBool(berr) => {
                    let message = format!(
                        "Bool parsing error \"{}\" in csv column {}",
                        berr,
                        deserialize_error
                            .field()
                            .map_or_else(|| "?".into(), |f| (f + 1).to_string())
                    );

                    Self {
                        kind: MeshErrorKind::GenericCSV { msg: message },
                        span: e.position().into(),
                    }
                }
            },
            csv::ErrorKind::Utf8 { err, .. } => Self {
                kind: MeshErrorKind::UTF8 {
                    column: Some(err.field() as u64 + 1),
                },
                span: e.position().into(),
            },
            _ => Self {
                kind: MeshErrorKind::UnknownCSV,
                span: e.position().into(),
            },
        }
    }
}
