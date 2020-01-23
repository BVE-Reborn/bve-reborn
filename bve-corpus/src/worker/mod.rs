use crate::{File, FileKind, FileResult, SharedData};
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Receiver, Sender};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

pub struct WorkerThread {
    pub handle: JoinHandle<()>,
    pub last_respond: Arc<AtomicCell<Instant>>,
    pub last_file: Arc<Mutex<PathBuf>>,
}

pub fn create_worker_thread(
    job_source: &Receiver<File>,
    result_sink: &Sender<FileResult>,
    shared: &Arc<SharedData>,
) -> WorkerThread {
    let last_respond: Arc<AtomicCell<Instant>> = Arc::new(AtomicCell::new(Instant::now()));
    let last_file = Arc::new(Mutex::new(PathBuf::new()));
    let handle = {
        let job_source = job_source.clone();
        let result_sink = result_sink.clone();
        let shared = Arc::clone(shared);
        let last_respond = Arc::clone(&last_respond);
        let last_file = Arc::clone(&last_file);
        std::thread::spawn(move || processing_loop(job_source, result_sink, shared, last_respond, last_file))
    };
    WorkerThread {
        handle,
        last_respond,
        last_file,
    }
}

fn processing_loop(
    job_source: Receiver<File>,
    result_sink: Sender<FileResult>,
    shared: Arc<SharedData>,
    last_respond: Arc<AtomicCell<Instant>>,
    last_file: Arc<Mutex<PathBuf>>,
) {
    while let Ok(file) = job_source.recv() {
        *last_file.lock().unwrap() = file.path.clone();
        let result = match file.kind {
            FileKind::RouteRw => {
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
            _ => {}
        };
        std::thread::sleep(std::time::Duration::from_millis(1));
        shared.total.finished.fetch_add(1, Ordering::SeqCst);
    }
}
