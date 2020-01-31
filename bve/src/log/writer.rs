use crate::log::common::*;
use crate::log::{Message, SerializationMethod};
use crossbeam::Receiver;
use num_traits::FromPrimitive;
use std::io::{BufWriter, Write};
use tracing::Id;

pub(super) fn run_writer(receiver: &Receiver<Command>, dest: impl Write + Send, method: SerializationMethod) {
    let mut buffered = BufWriter::new(dest);

    while let Ok(message) = receiver.recv() {
        let to_serialize = match message.data {
            CommandData::CreateSpan { id, metadata, data } => Message::CreateSpan {
                id: id.into_u64(),
                name: metadata.name().to_string(),
                module: metadata.module_path().map_or_else(String::new, str::to_string),
                file: metadata.file().map_or_else(String::new, str::to_string),
                line: metadata.line().and_then(u16::from_u32).unwrap_or_else(u16::max_value),
                severity: metadata.level().into(),
                values: data,
            },
            CommandData::RecordRelationship { parent, child } => Message::SpanParent {
                parent: parent.into_u64(),
                child: child.into_u64(),
            },
            CommandData::RecordSpanData { id, data } => Message::SpanData {
                id: id.into_u64(),
                values: data,
            },
            CommandData::Event {
                span_id,
                metadata,
                data,
            } => Message::Event {
                span_id: span_id.as_ref().map(Id::into_u64),
                file: metadata.file().map_or_else(String::new, str::to_string),
                line: metadata.line().and_then(u16::from_u32).unwrap_or_else(u16::max_value),
                severity: metadata.level().into(),
                message: data.get("message").map(|v| format!("{}", v)),
                values: data,
            },
        };

        method.serialize(&mut buffered, &to_serialize);

        buffered.flush().expect("Flushing to log destination failed");
    }
}
