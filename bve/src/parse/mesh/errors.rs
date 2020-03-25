use crate::l10n::ForceEnglish;
use crate::localize;
use crate::parse::{Span, UserError, UserErrorCategory};

/// A warning in the parsing or evaluation of a mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshWarning {
    /// Info about the exact error.
    pub kind: MeshWarningKind,
    /// Location of the error within the file.
    pub location: Span,
}

/// Enum representing various types of warnings encountered when parsing meshes
#[derive(Debug, Clone, PartialEq)]
pub enum MeshWarningKind {
    /// Instruction no longer does anything anymore
    UselessInstruction { name: String },
}

impl UserError for MeshWarning {
    fn category(&self) -> UserErrorCategory {
        UserErrorCategory::Warning
    }

    fn line(&self) -> u64 {
        self.location.line.unwrap_or(0)
    }

    fn description(&self, en: ForceEnglish) -> String {
        match &self.kind {
            MeshWarningKind::UselessInstruction { name } => {
                localize!(@en, "mesh-warning-useless-instruction", "name" -> name.as_str())
            }
        }
    }
}

/// A single error in the parsing or evaluation of a mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshError {
    /// Info about the exact error.
    pub kind: MeshErrorKind,
    /// Location of the error within the file.
    pub location: Span,
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
    /// Unrecognized instruction
    UnknownInstruction {
        /// Instruction that is not recognized
        name: String,
    },
    /// Handled CSV error
    GenericCSV {
        /// Message provided by CSV Library
        msg: String,
        /// Message in english
        msg_english: String,
    },
    /// Unknown csv error
    UnknownCSV,
}

impl UserError for MeshError {
    fn category(&self) -> UserErrorCategory {
        UserErrorCategory::Error
    }

    fn line(&self) -> u64 {
        self.location.line.unwrap_or(0)
    }

    fn description(&self, en: ForceEnglish) -> String {
        match &self.kind {
            MeshErrorKind::UTF8 { column } => localize!(@en, "mesh-error-utf8", "column" -> column.unwrap_or(0)),
            MeshErrorKind::OutOfBounds { idx } => localize!(@en, "mesh-error-out-of-bounds", "idx" -> *idx),
            MeshErrorKind::UnknownInstruction { name } => {
                localize!(@en, "mesh-error-unknown-instruction", "name" -> name.as_str())
            }
            MeshErrorKind::GenericCSV { msg, msg_english } => {
                if en == ForceEnglish::English { msg_english } else { msg }.clone()
            }
            MeshErrorKind::UnknownCSV => localize!(@en, "mesh-unknown-csv"),
        }
    }
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
                    location: e.position().into(),
                },
                csv::DeserializeErrorKind::Message(msg) | csv::DeserializeErrorKind::Unsupported(msg) => Self {
                    kind: MeshErrorKind::GenericCSV {
                        msg: msg.clone(),
                        msg_english: msg.clone(),
                    },
                    location: e.position().into(),
                },
                csv::DeserializeErrorKind::UnexpectedEndOfRow => Self {
                    kind: MeshErrorKind::GenericCSV {
                        msg: localize!("csv-unexpected-end-of-row"),
                        msg_english: localize!(english, "csv-unexpected-end-of-row"),
                    },
                    location: e.position().into(),
                },
                csv::DeserializeErrorKind::ParseFloat(f_err) => {
                    let column = deserialize_error
                        .field()
                        .map_or_else(|| "?".into(), |f| (f + 1).to_string());
                    let formatted_err = format!("{}", f_err);
                    let msg = localize!("csv-float-parsing-error", "error" -> formatted_err.clone(), "column" -> column.clone());
                    let msg_english =
                        localize!(english, "csv-float-parsing-error", "error" -> formatted_err, "column" -> column);

                    Self {
                        kind: MeshErrorKind::GenericCSV { msg, msg_english },
                        location: e.position().into(),
                    }
                }
                csv::DeserializeErrorKind::ParseInt(i_err) => {
                    let column = deserialize_error
                        .field()
                        .map_or_else(|| "?".into(), |f| (f + 1).to_string());
                    let formatted_err = format!("{}", i_err);
                    let msg = localize!("csv-int-parsing-error", "error" -> formatted_err.clone(), "column" -> column.clone());
                    let msg_english =
                        localize!(english, "csv-int-parsing-error", "error" -> formatted_err, "column" -> column);

                    Self {
                        kind: MeshErrorKind::GenericCSV { msg, msg_english },
                        location: e.position().into(),
                    }
                }
                csv::DeserializeErrorKind::ParseBool(b_err) => {
                    let column = deserialize_error
                        .field()
                        .map_or_else(|| "?".into(), |f| (f + 1).to_string());
                    let formatted_err = format!("{}", b_err);
                    let msg = localize!("csv-bool-parsing-error", "error" -> formatted_err.clone(), "column" -> column.clone());
                    let msg_english =
                        localize!(english, "csv-bool-parsing-error", "error" -> formatted_err, "column" -> column);

                    Self {
                        kind: MeshErrorKind::GenericCSV { msg, msg_english },
                        location: e.position().into(),
                    }
                }
            },
            csv::ErrorKind::Utf8 { err, .. } => Self {
                kind: MeshErrorKind::UTF8 {
                    column: Some(err.field() as u64 + 1),
                },
                location: e.position().into(),
            },
            _ => Self {
                kind: MeshErrorKind::UnknownCSV,
                location: e.position().into(),
            },
        }
    }
}
