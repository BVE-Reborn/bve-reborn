use crate::log::common::*;
use crate::log::Message;
use crossbeam::Receiver;
use std::io::{BufWriter, Write};

pub(super) fn run_writer(receiver: Receiver<Command>, dest: impl Write + Send) {
    let mut buffered = BufWriter::new(dest);

    while let Ok(message) = receiver.recv() {
        match message.data {
            CommandData::CreateSpan { id, metadata, data } => {
                Message::CreateSpan {
                    id: id.into_u64(),
                    name: metadata.name().to_string(),
                    module: metadata.module_path().map(str::to_string).unwrap_or_else(String::new),
                    file: metadata.file().map(str::to_string).unwrap_or_else(String::new),
                    line: metadata.line().map(|v| v as u16).unwrap_or_else(u16::max_value),
                    severity: metadata.level().into(),
                    values: data.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
                };
            }
            CommandData::RecordRelationship { parent, child } => {
                writeln!(buffered, "Span {} is parent of {}", parent.into_u64(), child.into_u64());
            }
            CommandData::RecordSpanData { id, data } => {
                writeln!(buffered, "Span {} has data: {:?}", id.into_u64(), data);
            }
            CommandData::Event {
                span_id,
                metadata,
                data,
            } => {
                let span_id_u64 = span_id.map(|v| v.into_u64()).unwrap_or(0);
                writeln!(
                    buffered,
                    "Event! Span: {}. Name: {}. Data: {:?}",
                    span_id_u64,
                    metadata.name(),
                    data
                );
            }
        }
        buffered.flush();
    }
}
