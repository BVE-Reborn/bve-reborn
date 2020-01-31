use crate::log::writer::run_writer;
use crate::log::*;
use crossbeam::{unbounded, Sender};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use tracing::span::{Attributes, Record};
use tracing::Id;
use tracing_core::field::Visit;
use tracing_core::{Event, Field, Metadata};

thread_local! {
    static CURRENT_SPAN: RefCell<Option<Id>> = RefCell::new(None);
}

pub struct Subscriber {
    current_span_id: AtomicU64,
    background_sender: Sender<Command>, // Must be dropped before thread
    background_thread: JoinHandle<()>,
}

impl Subscriber {
    pub fn new(dest: impl Write + Send + 'static) -> Self {
        let (sender, receiver) = unbounded();

        let handle = std::thread::spawn(move || {
            run_writer(receiver, dest);
        });

        Self {
            current_span_id: AtomicU64::new(1),
            background_thread: handle,
            background_sender: sender,
        }
    }

    pub fn terminate(self) {
        drop(self.background_sender);
    }
}

impl tracing::Subscriber for Subscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        true
    }

    // noinspection DuplicatedCode
    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let id = Id::from_u64(self.current_span_id.fetch_add(1, Ordering::Relaxed));

        // Record all members with a visitor
        let mut visitor = RecordVisitor::new();
        span.record(&mut visitor);

        // Send the creation of the span to the background thread
        self.background_sender.send(Command::from_data(CommandData::CreateSpan {
            id: id.clone(),
            name: span.metadata().name(),
            data: visitor.into_data(),
        }));

        // Determine if this span has a parent, and if it does, get the ID
        let parent = if span.is_contextual() {
            CURRENT_SPAN.with(|v| v.borrow().clone())
        } else {
            span.parent().map(Id::clone)
        };

        // If there is a parent, explain the relationship to the backend
        if let Some(parent_id) = parent {
            self.background_sender
                .send(Command::from_data(CommandData::RecordRelationship {
                    parent: parent_id,
                    child: id.clone(),
                }));
        }

        id
    }

    fn record(&self, span: &Id, values: &Record<'_>) {
        // Record all members with a visitor
        let mut visitor = RecordVisitor::new();
        values.record(&mut visitor);

        // Send the new information to the backend
        self.background_sender
            .send(Command::from_data(CommandData::RecordSpanData {
                id: span.clone(),
                data: visitor.into_data(),
            }));
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        self.background_sender
            .send(Command::from_data(CommandData::RecordRelationship {
                parent: follows.clone(),
                child: span.clone(),
            }));
    }

    // noinspection DuplicatedCode
    fn event(&self, event: &Event<'_>) {
        // Record all members with a visitor
        let mut visitor = RecordVisitor::new();
        event.record(&mut visitor);

        // Determine if this event has a span, and if it does, get the ID
        let span = if event.is_contextual() {
            CURRENT_SPAN.with(|v| v.borrow().clone())
        } else {
            event.parent().map(Id::clone)
        };

        // Send event
        self.background_sender.send(Command::from_data(CommandData::Event {
            span_id: span,
            data: visitor.into_data(),
        }));
    }

    fn enter(&self, span: &Id) {
        CURRENT_SPAN.with(|v| v.replace(Some(span.clone())));
    }

    fn exit(&self, span: &Id) {
        CURRENT_SPAN.with(|v| v.replace(None));
    }
}

struct RecordVisitor {
    data: HashMap<&'static str, Data>,
}

impl RecordVisitor {
    fn new() -> Self {
        Self {
            data: HashMap::default(),
        }
    }

    fn into_data(self) -> HashMap<&'static str, Data> {
        self.data
    }
}

impl Visit for RecordVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.data.insert(field.name(), Data::I64(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.data.insert(field.name(), Data::U64(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.data.insert(field.name(), Data::Bool(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.data.insert(field.name(), Data::String(value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.data.insert(field.name(), Data::String(format!("{:?}", value)));
    }
}
