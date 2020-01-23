use crate::*;
use crossbeam::Sender;

fn enumerate(path: impl AsRef<Path>, mut func: impl FnMut(PathBuf, DirEntry) -> ()) {
    for entry in WalkDir::new(path.as_ref()).follow_links(true).same_file_system(false) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let mut file_path = path.as_ref().to_path_buf();
                file_path.push(entry.path());
                func(file_path, entry);
            }
        }
    }
}

fn add_file(file_sink: &Sender<File>, shared: &SharedData, stats: &Stats, file: File) {
    file_sink.send(file).unwrap();
    shared.total.total.fetch_add(1, Ordering::SeqCst);
    stats.total.fetch_add(1, Ordering::SeqCst);
}

pub fn enumerate_all_files(options: Options, file_sink: Sender<File>, shared: Arc<SharedData>) {
    let mut entry_func = |path: PathBuf, _entry: DirEntry| {
        let shared = shared.as_ref();
        match path.file_name().map(|v| v.to_string_lossy().to_lowercase()) {
            p if p == Some("train.dat".into()) => add_file(
                &file_sink,
                shared,
                &shared.train_dat,
                File {
                    path,
                    kind: FileKind::TrainDat,
                },
            ),
            p if p == Some("train.xml".into()) => add_file(
                &file_sink,
                shared,
                &shared.train_xml,
                File {
                    path,
                    kind: FileKind::TrainXML,
                },
            ),
            p if p == Some("extensions.cfg".into()) => add_file(
                &file_sink,
                shared,
                &shared.extensions_cfg,
                File {
                    path,
                    kind: FileKind::ExtensionsCfg,
                },
            ),
            p if p == Some("panel.cfg".into()) => add_file(
                &file_sink,
                shared,
                &shared.panel_cfg,
                File {
                    path,
                    kind: FileKind::PanelCfg,
                },
            ),
            p if p == Some("panel2.cfg".into()) => add_file(
                &file_sink,
                shared,
                &shared.panel_cfg2,
                File {
                    path,
                    kind: FileKind::PanelCfg2,
                },
            ),
            p if p == Some("sound.cfg".into()) => add_file(
                &file_sink,
                shared,
                &shared.sound_cfg,
                File {
                    path,
                    kind: FileKind::SoundCfg,
                },
            ),
            p if p == Some("ats.cfg".into()) => add_file(
                &file_sink,
                shared,
                &shared.ats_cfg,
                File {
                    path,
                    kind: FileKind::AtsCfg,
                },
            ),
            _ => match path.extension().map(|v| v.to_string_lossy().to_lowercase()) {
                ext if ext == Some("b3d".into()) => add_file(
                    &file_sink,
                    shared,
                    &shared.model_b3d,
                    File {
                        path,
                        kind: FileKind::ModelB3d,
                    },
                ),
                ext if ext == Some("csv".into()) => add_file(
                    &file_sink,
                    shared,
                    &shared.model_csv,
                    File {
                        path,
                        kind: FileKind::ModelCsv,
                    },
                ),
                ext if ext == Some("animated".into()) => add_file(
                    &file_sink,
                    shared,
                    &shared.model_animated,
                    File {
                        path,
                        kind: FileKind::ModelAnimated,
                    },
                ),
                Some(_ext) => {
                    // unrecognized
                }
                _ => {}
            },
        }
    };

    let mut path = options.root_path.clone();
    path.push("LegacyContent");
    path.push("Railway");
    path.push("Object");

    enumerate(path, &mut entry_func);

    let mut path = options.root_path.clone();
    path.push("LegacyContent");
    path.push("Train");

    enumerate(path, &mut entry_func);

    let mut path = options.root_path.clone();
    path.push("LegacyContent");
    path.push("Railway");
    path.push("Route");

    enumerate(path, |path: PathBuf, _entry| {
        let shared = shared.as_ref();
        match path.extension().map(|v| v.to_string_lossy().to_lowercase()) {
            ext if ext == Some("csv".into()) => add_file(
                &file_sink,
                shared,
                &shared.route_csv,
                File {
                    path,
                    kind: FileKind::RouteCsv,
                },
            ),
            ext if ext == Some("rw".into()) => add_file(
                &file_sink,
                shared,
                &shared.route_rw,
                File {
                    path,
                    kind: FileKind::RouteRw,
                },
            ),
            Some(_ext) => {
                // unrecognized
            }
            _ => {}
        }
    });

    shared.fully_loaded.store(true, Ordering::SeqCst);
}
