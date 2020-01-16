#[derive(Debug, Clone, PartialEq)]
pub struct MeshError {
    pub kind: MeshErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MeshErrorKind {
    UTF8 { column: Option<u64> },
    OutOfBounds { idx: usize },
    DeprecatedInstruction { name: String },
    UnknownInstruction { name: String },
    GenericCSV { msg: String },
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Span {
    pub line: Option<u64>,
}

impl<'a> From<Option<&'a csv::Position>> for Span {
    #[must_use]
    fn from(p: Option<&'a csv::Position>) -> Self {
        Self {
            line: p.map(csv::Position::line),
        }
    }
}
