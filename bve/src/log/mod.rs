use std::collections::HashMap;
use std::thread::ThreadId;
use std::time::Instant;
pub use subscriber::*;
use tracing::Id;

mod subscriber;
mod writer;

#[derive(Debug, Clone)]
struct Command {
    time: Instant,
    thread: ThreadId,
    data: CommandData,
}

impl Command {
    fn from_data(data: CommandData) -> Self {
        Self {
            time: Instant::now(),
            thread: std::thread::current().id(),
            data,
        }
    }
}

#[derive(Debug, Clone)]
enum CommandData {
    CreateSpan {
        id: Id,
        name: &'static str,
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
        data: HashMap<&'static str, Data>,
    },
}

#[derive(Debug, Clone)]
enum Data {
    U64(u64),
    I64(i64),
    Bool(bool),
    String(String),
}
