use crate::enumeration::enumerate_all_files;
use crossbeam::channel::unbounded;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
pub use options::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

mod enumeration;
mod options;
mod thread_kill;
mod worker;

#[derive(Debug, Default)]
pub struct Stats {
    finished: AtomicU64,
    total: AtomicU64,
}

pub struct File {
    path: PathBuf,
    kind: FileKind,
}

enum FileKind {
    AtsCfg,
    ExtensionsCfg,
    ModelAnimated,
    ModelB3d,
    ModelCsv,
    PanelCfg,
    PanelCfg2,
    RouteCsv,
    RouteRw,
    SoundCfg,
    TrainDat,
    TrainXML,
}

#[derive(Debug, Default)]
pub struct SharedData {
    total: Stats,
    ats_cfg: Stats,
    extensions_cfg: Stats,
    model_animated: Stats,
    model_b3d: Stats,
    model_csv: Stats,
    panel_cfg: Stats,
    panel_cfg2: Stats,
    route_csv: Stats,
    route_rw: Stats,
    sound_cfg: Stats,
    train_dat: Stats,
    train_xml: Stats,

    fully_loaded: AtomicBool,
}

fn main() {
    let options: Options = Options::from_args();

    let shared = Arc::new(SharedData::default());
    let (sender, receiver) = unbounded();

    // Progress bars
    let mp = MultiProgress::new();
    let style = ProgressStyle::default_spinner()
        .template("Total: {wide_bar} {pos:>6}/{len:6} {msg}")
        .progress_chars("##-");

    let total_progress = mp.add(ProgressBar::new(0).with_style(style.clone()));

    let sending = {
        let shared = Arc::clone(&shared);
        let options = options.clone();
        std::thread::spawn(move || enumerate_all_files(options, sender, shared))
    };

    let progress_thread = std::thread::spawn(move || mp.join().unwrap());

    while shared.fully_loaded.load(Ordering::SeqCst) == false {
        total_progress.set_length(shared.total.total.load(Ordering::SeqCst));
        std::thread::sleep(Duration::from_millis(2));
    }

    total_progress.finish();

    sending.join().unwrap();
    progress_thread.join().unwrap();

    dbg!(shared);
}
