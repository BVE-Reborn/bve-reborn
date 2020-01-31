pub use crate::log::common::Data as MessageValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use tracing_core::Level;

#[derive(Clone, Debug)]
pub enum SerializationMethod {
    Bincode,
    Json,
    JsonPretty,
}

impl SerializationMethod {
    pub fn serialize(&self, writer: &mut impl Write, message: &Message) {
        match self {
            Self::Bincode => bincode::serialize_into(writer, message).expect("Bincode serialization failed"),
            Self::Json => {
                serde_json::to_writer(&mut *writer, message).expect("Json serialization failed");
                writeln!(writer, "");
            }
            Self::JsonPretty => {
                serde_json::to_writer_pretty(&mut *writer, message).expect("Json serialization failed");
                writeln!(writer, "");
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    CreateSpan {
        id: u64,
        name: String,
        module: String,
        file: String,
        line: u16,
        severity: Severity,
        values: HashMap<String, MessageValue>,
    },
    SpanData {
        id: u64,
        values: HashMap<String, MessageValue>,
    },
    SpanParent {
        parent: u64,
        child: u64,
    },
    Event {
        span_id: Option<u64>,
        file: String,
        line: u16,
        severity: Severity,
        message: Option<String>,
        values: HashMap<String, MessageValue>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Severity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<Level> for Severity {
    #[inline]
    fn from(l: Level) -> Self {
        From::from(&l)
    }
}

impl From<&Level> for Severity {
    #[inline]
    fn from(l: &Level) -> Self {
        if *l == Level::ERROR {
            Self::Error
        } else if *l == Level::WARN {
            Self::Warn
        } else if *l == Level::INFO {
            Self::Info
        } else if *l == Level::DEBUG {
            Self::Debug
        } else {
            Self::Trace
        }
    }
}
