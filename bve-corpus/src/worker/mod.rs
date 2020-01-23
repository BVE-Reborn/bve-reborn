use crate::{File, FileKind, FileResult, SharedData};
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Receiver, Sender};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

pub struct WorkerThread {
    pub handle: JoinHandle<()>,
    pub last_respond: Arc<AtomicCell<Instant>>,
}

pub fn create_worker_thread(
    job_source: &Receiver<File>,
    result_sink: &Sender<FileResult>,
    shared: &Arc<SharedData>,
) -> WorkerThread {
    let last_respond: Arc<AtomicCell<Instant>> = Arc::new(AtomicCell::new(Instant::now()));
    let handle = {
        let job_source = job_source.clone();
        let result_sink = result_sink.clone();
        let shared = Arc::clone(shared);
        let last_respond = Arc::clone(&last_respond);
        std::thread::spawn(move || processing_loop(job_source, result_sink, shared, last_respond))
    };
    WorkerThread { handle, last_respond }
}

fn processing_loop(
    job_source: Receiver<File>,
    result_sink: Sender<FileResult>,
    shared: Arc<SharedData>,
    last_respond: Arc<AtomicCell<Instant>>,
) {
    while let Ok(file) = job_source.recv() {
        std::thread::sleep(std::time::Duration::from_millis(1));
        let result = match file.kind {
            _ => {}
        };
        shared.total.finished.fetch_add(1, Ordering::SeqCst);
    }
}
