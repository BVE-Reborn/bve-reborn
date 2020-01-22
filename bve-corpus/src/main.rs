use crossbeam::channel::unbounded;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

mod enumeration;
mod options;

use crate::enumeration::enumerate_all_files;
pub use options::*;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Stats {
    finished: AtomicUsize,
    total: AtomicUsize,
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
}

fn main() {
    let options: Options = Options::from_args();

    let shared = Arc::new(SharedData::default());
    let (sender, receiver) = unbounded();

    let sending = {
        let shared = Arc::clone(&shared);
        let options = options.clone();
        std::thread::spawn(move || enumerate_all_files(options, sender, shared))
    };

    sending.join().unwrap();

    dbg!(shared);
}
