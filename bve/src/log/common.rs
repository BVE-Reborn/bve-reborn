use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::thread::ThreadId;
use std::time::Instant;
use tracing::Id;
use tracing_core::Metadata;

#[derive(Debug, Clone)]
pub struct Command {
    pub time: Instant,
    pub thread: ThreadId,
    pub data: CommandData,
}

impl Command {
    pub fn from_data(data: CommandData) -> Self {
        Self {
            time: Instant::now(),
            thread: std::thread::current().id(),
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CommandData {
    CreateSpan {
        id: Id,
        metadata: &'static Metadata<'static>,
        data: HashMap<String, Data>, /* This is owned because it needs to be owned in the writer serialization
                                      * structure. */
    },
    RecordSpanData {
        id: Id,
        data: HashMap<String, Data>, /* This is owned because it needs to be owned in the writer serialization
                                      * structure. */
    },
    RecordRelationship {
        parent: Id,
        child: Id,
    },
    Event {
        span_id: Option<Id>,
        metadata: &'static Metadata<'static>,
        data: HashMap<String, Data>, /* This is owned because it needs to be owned in the writer serialization
                                      * structure. */
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Data {
    U64(u64),
    I64(i64),
    Bool(bool),
    String(String),
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::U64(v) => Display::fmt(v, f),
            Self::I64(v) => Display::fmt(v, f),
            Self::Bool(v) => Display::fmt(v, f),
            Self::String(v) => Display::fmt(v, f),
        }
    }
}
