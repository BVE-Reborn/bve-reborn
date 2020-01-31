use crate::log::*;
use crossbeam::Receiver;
use std::io::{BufWriter, Write};

pub(super) fn run_writer(receiver: Receiver<Command>, dest: impl Write + Send) {
    let mut buffered = BufWriter::new(dest);

    while let Ok(message) = receiver.recv() {
        match message.data {
            CommandData::CreateSpan { id, name, data } => {
                writeln!(buffered, "Creating span {}: {}. Data: {:?}", id.into_u64(), name, data);
            }
            CommandData::RecordRelationship { parent, child } => {
                writeln!(buffered, "Span {} is parent of {}", parent.into_u64(), child.into_u64());
            }
            CommandData::RecordSpanData { id, data } => {
                writeln!(buffered, "Span {} has data: {:?}", id.into_u64(), data);
            }
            CommandData::Event { span_id, data } => {
                let span_id_u64 = span_id.map(|v| v.into_u64()).unwrap_or(0);
                writeln!(buffered, "Event! Span: {}. Data: {:?}", span_id_u64, data);
            }
        }
        buffered.flush();
    }
}
