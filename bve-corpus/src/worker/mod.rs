use crate::{
    panic::{PANIC, USE_DEFAULT_PANIC_HANLDER},
    File, FileKind, FileResult, ParseResult, SharedData,
};
use async_std::task::block_on;
use bve::{
    filesystem::read_convert_utf8,
    panic_log,
    parse::{
        animated::ParsedAnimatedObject,
        ats_cfg::ParsedAtsConfig,
        extensions_cfg::ParsedExtensionsCfg,
        mesh::{ParsedStaticObjectB3D, ParsedStaticObjectCSV},
        panel1_cfg::ParsedPanel1Cfg,
        panel2_cfg::ParsedPanel2Cfg,
        sound_cfg::ParsedSoundCfg,
        train_dat::ParsedTrainDat,
        FileAwareFileParser, ParserResult, UserError,
    },
};
use crossbeam_channel::{Receiver, Sender};
use crossbeam_utils::atomic::AtomicCell;
use log::warn;
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Instant,
};

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
        std::thread::spawn(move || processing_loop(&job_source, &result_sink, &shared, &last_respond, &last_file))
    };
    WorkerThread {
        handle,
        last_respond,
        last_file,
    }
}

fn read_from_file(filename: impl AsRef<Path>) -> String {
    match block_on(read_convert_utf8(filename.as_ref())) {
        Ok(s) => s,
        Err(err) => {
            panic_log!("Loading error: {:?}", err);
        }
    }
}

fn run_parser<P: FileAwareFileParser>(path: &str, input: &str, counter: &AtomicU64) -> ParseResult {
    let ParserResult { warnings, errors, .. } = block_on(P::file_aware_parse_from(&[path], path, input));

    counter.fetch_add(1, Ordering::AcqRel);

    if warnings.is_empty() && errors.is_empty() {
        ParseResult::Success
    } else {
        ParseResult::Issues {
            warnings: warnings.iter().map(UserError::to_data).collect(),
            errors: errors.iter().map(UserError::to_data).collect(),
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
        let folder = file_ref.path.parent().unwrap().to_string_lossy();
        // Say that we're still alive
        last_respond.store(Instant::now());

        USE_DEFAULT_PANIC_HANLDER.with(|v| *v.borrow_mut() = false);
        let panicked = std::panic::catch_unwind(|| match &file_ref.kind {
            FileKind::AtsCfg => run_parser::<ParsedAtsConfig>(&folder, &file_contents, &shared.ats_cfg.finished),
            FileKind::ModelCsv => {
                run_parser::<ParsedStaticObjectCSV>(&folder, &file_contents, &shared.model_csv.finished)
            }
            FileKind::ModelB3d => {
                run_parser::<ParsedStaticObjectB3D>(&folder, &file_contents, &shared.model_b3d.finished)
            }
            FileKind::ModelAnimated => {
                run_parser::<ParsedAnimatedObject>(&folder, &file_contents, &shared.model_animated.finished)
            }
            FileKind::TrainDat => run_parser::<ParsedTrainDat>(&folder, &file_contents, &shared.train_dat.finished),
            FileKind::ExtensionsCfg => {
                run_parser::<ParsedExtensionsCfg>(&folder, &file_contents, &shared.extensions_cfg.finished)
            }
            FileKind::Panel1Cfg => run_parser::<ParsedPanel1Cfg>(&folder, &file_contents, &shared.panel1_cfg.finished),
            FileKind::Panel2Cfg => run_parser::<ParsedPanel2Cfg>(&folder, &file_contents, &shared.panel2_cfg.finished),
            FileKind::SoundCfg => run_parser::<ParsedSoundCfg>(&folder, &file_contents, &shared.sound_cfg.finished),
            _ => ParseResult::Success,
        });
        USE_DEFAULT_PANIC_HANLDER.with(|v| *v.borrow_mut() = true);

        let duration = Instant::now() - start;

        let result = match panicked {
            Ok(parse_result) => parse_result,
            Err(..) => PANIC.with(|v| {
                warn!("Panicked while parsing: {:?}", file_ref.path);

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

        result_sink.send(file_result).unwrap_or_else(|_| {
            panic_log!("Send error on file {}", file_path.display());
        });

        // Dump the total amount worked on
        shared.total.finished.fetch_add(1, Ordering::SeqCst);
    }
}
