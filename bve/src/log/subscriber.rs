use crate::log::{common::*, writer::run_writer, SerializationMethod};
use crossbeam::{bounded, Sender};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    io::Write,
    mem::ManuallyDrop,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};
use tracing::{
    span::{Attributes, Record},
    Id,
};
use tracing_core::{field::Visit, Event, Field, Level, Metadata};

thread_local! {
    static CURRENT_SPAN: RefCell<Option<Id>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct Subscriber {
    inner: Arc<SubscriberData>,
}

struct SubscriberData {
    current_span_id: AtomicU64,
    level: Level,
    background_sender: ManuallyDrop<Sender<Command>>, // Must be dropped before thread
    background_thread: ManuallyDrop<JoinHandle<()>>,  // Needs to be joined
    // Points from child to parent
    span_parents: Mutex<HashMap<u64, u64>>,
}

impl Subscriber {
    pub fn new(dest: impl Write + Send + 'static, level: Level, method: SerializationMethod) -> Self {
        // Use a bounded queue to prevent an influx of messages using all of memory
        let (sender, receiver) = bounded(1024_usize);

        let handle = std::thread::spawn(move || {
            run_writer(&receiver, dest, &method);
        });

        Self {
            inner: Arc::new(SubscriberData {
                current_span_id: AtomicU64::new(1),
                level,
                background_thread: ManuallyDrop::new(handle),
                background_sender: ManuallyDrop::new(sender),
                span_parents: Mutex::new(HashMap::new()),
            }),
        }
    }

    fn add_parent(&self, parent: Id, child: Id) {
        self.inner
            .span_parents
            .lock()
            .expect("Need to lock parental_relationship map")
            .insert(child.into_u64(), parent.into_u64());
        self.inner
            .background_sender
            .send(Command::from_data(CommandData::RecordRelationship { parent, child }))
            .expect("Cannot send parent to logger");
    }
}

impl Drop for SubscriberData {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.background_sender);
        }
        let thread = unsafe { ManuallyDrop::take(&mut self.background_thread) };
        thread.join().expect("Logging thread panicked");
    }
}

impl tracing::Subscriber for Subscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.inner.level <= *metadata.level()
    }

    // noinspection DuplicatedCode
    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let id = Id::from_u64(self.inner.current_span_id.fetch_add(1, Ordering::Relaxed));

        // Record all members with a visitor
        let mut visitor = RecordVisitor::new();
        span.record(&mut visitor);

        // Send the creation of the span to the background thread
        self.inner
            .background_sender
            .send(Command::from_data(CommandData::CreateSpan {
                id: id.clone(),
                metadata: span.metadata(),
                data: visitor.into_data(),
            }))
            .expect("Cannot send span to logger");

        // Determine if this span has a parent, and if it does, get the ID
        let parent = if span.is_contextual() {
            CURRENT_SPAN.with(|v| v.borrow().clone())
        } else {
            span.parent().map(Id::clone)
        };

        // If there is a parent, explain the relationship to the backend
        if let Some(parent_id) = parent {
            self.add_parent(parent_id, id.clone())
        }

        id
    }

    fn record(&self, span: &Id, values: &Record<'_>) {
        // Record all members with a visitor
        let mut visitor = RecordVisitor::new();
        values.record(&mut visitor);

        // Send the new information to the backend
        self.inner
            .background_sender
            .send(Command::from_data(CommandData::RecordSpanData {
                id: span.clone(),
                data: visitor.into_data(),
            }))
            .expect("Cannot send record to logger");
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        self.add_parent(follows.clone(), span.clone())
    }

    // noinspection DuplicatedCode
    fn event(&self, event: &Event<'_>) {
        if *event.metadata().level() > self.inner.level {
            return;
        }

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
        self.inner
            .background_sender
            .send(Command::from_data(CommandData::Event {
                span_id: span,
                metadata: event.metadata(),
                data: visitor.into_data(),
            }))
            .expect("Cannot send event to logger");
    }

    fn enter(&self, span: &Id) {
        CURRENT_SPAN.with(|v| v.replace(Some(span.clone())));
    }

    fn exit(&self, span: &Id) {
        let parent = self
            .inner
            .span_parents
            .lock()
            .expect("Need to lock parental_relationship map")
            .get(&span.into_u64())
            .map(|v| Id::from_u64(*v));
        CURRENT_SPAN.with(|v| v.replace(parent));
    }

    fn try_close(&self, id: Id) -> bool {
        self.inner
            .span_parents
            .lock()
            .expect("Need to lock parental_relationship map")
            .remove(&id.into_u64());
        true
    }
}

struct RecordVisitor {
    data: HashMap<String, Data>, /* This is owned because it needs to be owned in the writer serialization
                                  * structure. */
}

impl RecordVisitor {
    fn new() -> Self {
        Self {
            data: HashMap::default(),
        }
    }

    #[allow(clippy::missing_const_for_fn)] // flat out wrong
    fn into_data(self) -> HashMap<String, Data> {
        self.data
    }
}

impl Visit for RecordVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.data.insert(field.name().to_string(), Data::I64(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.data.insert(field.name().to_string(), Data::U64(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.data.insert(field.name().to_string(), Data::Bool(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.data
            .insert(field.name().to_string(), Data::String(value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.data
            .insert(field.name().to_string(), Data::String(format!("{:?}", value)));
    }
}
