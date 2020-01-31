use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        data: HashMap<&'static str, Data>,
    },
    RecordSpanData {
        id: Id,
        data: HashMap<&'static str, Data>,
    },
    RecordRelationship {
        parent: Id,
        child: Id,
    },
    Event {
        span_id: Option<Id>,
        metadata: &'static Metadata<'static>,
        data: HashMap<&'static str, Data>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Data {
    U64(u64),
    I64(i64),
    Bool(bool),
    String(String),
}
