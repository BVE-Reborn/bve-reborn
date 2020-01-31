pub use crate::log::common::Data as MessageValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing_core::Level;

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
        span_id: u64,
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
            Severity::Error
        } else if *l == Level::WARN {
            Severity::Warn
        } else if *l == Level::INFO {
            Severity::Info
        } else if *l == Level::DEBUG {
            Severity::Debug
        } else {
            Severity::Trace
        }
    }
}
