use crate::panic::{PANIC, USE_DEFAULT_PANIC_HANLDER};
use crate::{File, FileKind, FileResult, ParseResult, SharedData};
use bve::filesystem::read_convert_utf8;
use bve::parse::animated::parse_animated_file;
use bve::parse::ats_cfg::parse_ats_cfg;
use bve::parse::extensions_cfg::parse_extensions_cfg;
use bve::parse::kvp::parse_kvp_file;
use bve::parse::mesh::{mesh_from_str, FileType, MeshErrorKind, ParsedStaticObject};
use bve::parse::panel1_cfg::parse_panel1_cfg;
use bve::parse::train_dat::parse_train_dat;
use core::panicking::panic;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Receiver, Sender};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;
use std::fmt::Debug;
use std::fs::read_to_string;
use std::io::Write;
use std::path::{Path, PathBuf};
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
        bve::concurrency::spawn(move || processing_loop(&job_source, &result_sink, &shared, &last_respond, &last_file))
    };
    WorkerThread {
        handle,
        last_respond,
        last_file,
    }
}

fn read_from_file(filename: impl AsRef<Path>) -> String {
    match read_convert_utf8(filename) {
        Ok(s) => s,
        Err(err) => {
            println!("Path Error: {:?}", err);
            panic!("Path Error: {:?}", err)
        }
    }
}

fn success_or_errors<E>(errors: Vec<E>) -> ParseResult
where
    E: Debug,
{
    if errors.is_empty() {
        ParseResult::Success
    } else {
        ParseResult::Errors {
            count: errors.len() as u64,
            error: anyhow::Error::msg(errors.into_iter().map(|v| format!("{:?}", v)).join(",")),
        }
    }
}

fn processing_loop(
    job_source: &Receiver<File>,
    result_sink: &Sender<FileResult>,
    shared: &SharedData,
    last_respond: &AtomicCell<Instant>,
    last_file: &Mutex<PathBuf>,
) {
    while let Ok(file) = job_source.recv() {
        // Set last file to our current file
        *last_file.lock().unwrap() = file.path.clone();
        // Say that we're alive
        last_respond.store(Instant::now());
        // Get beginning time
        let start = Instant::now();

        let file_ref = &file;

        // File reading isn't part of the operation.
        let file_contents = read_from_file(&file_ref.path);
        // Say that we're still alive
        last_respond.store(Instant::now());

        USE_DEFAULT_PANIC_HANLDER.with(|v| *v.borrow_mut() = false);
        let panicked = std::panic::catch_unwind(|| match &file_ref.kind {
            FileKind::AtsCfg => {
                let (_animated, warnings) = parse_ats_cfg(&file_contents);

                shared.ats_cfg.finished.fetch_add(1, Ordering::AcqRel);

                success_or_errors(warnings)
            }
            FileKind::ModelCsv => {
                let ParsedStaticObject { errors, .. } = mesh_from_str(&file_contents, FileType::CSV);

                shared.model_csv.finished.fetch_add(1, Ordering::AcqRel);

                success_or_errors(errors)
            }
            FileKind::ModelB3d => {
                let ParsedStaticObject { errors, .. } = mesh_from_str(&file_contents, FileType::B3D);

                shared.model_b3d.finished.fetch_add(1, Ordering::AcqRel);

                success_or_errors(errors)
            }
            FileKind::ModelAnimated => {
                let (_animated, warnings) = parse_animated_file(&file_contents);

                shared.model_animated.finished.fetch_add(1, Ordering::AcqRel);

                success_or_errors(warnings)
            }
            FileKind::TrainDat => {
                let (_parsed, warnings) = parse_train_dat(&file_contents);

                shared.train_dat.finished.fetch_add(1, Ordering::AcqRel);

                success_or_errors(warnings)
            }
            FileKind::ExtensionsCfg => {
                let (_parsed, warnings) = parse_extensions_cfg(&file_contents);

                shared.extensions_cfg.finished.fetch_add(1, Ordering::AcqRel);

                success_or_errors(warnings)
            }
            FileKind::PanelCfg => {
                let (_parsed, warnings) = parse_panel1_cfg(&file_contents);

                shared.panel_cfg.finished.fetch_add(1, Ordering::AcqRel);

                success_or_errors(warnings)
            }
            _ => ParseResult::Success,
        });
        USE_DEFAULT_PANIC_HANLDER.with(|v| *v.borrow_mut() = true);

        let duration = Instant::now() - start;

        let result = match panicked {
            Ok(parse_result) => parse_result,
            Err(..) => PANIC.with(|v| {
                let stderr = std::io::stderr();
                let path_str = format!("Panicked while parsing: {:?}\n", file_ref.path);
                let mut stderr_guard = stderr.lock();
                stderr_guard.write_all(path_str.as_bytes()).unwrap();
                drop(stderr_guard);

                let m = &mut *v.borrow_mut();
                let cause = m.take().unwrap_or_else(String::default);
                ParseResult::Panic { cause }
            }),
        };

        let file_path = file.path;

        let file_result = FileResult {
            path: file_path.clone(),
            kind: file.kind,
            result,
            _duration: duration,
        };

        result_sink
            .send(file_result)
            .unwrap_or_else(|_| panic!("Send error on file {}", file_path.display()));

        // Dump the total amount worked on
        shared.total.finished.fetch_add(1, Ordering::SeqCst);
    }
}
