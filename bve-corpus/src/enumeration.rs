use crate::*;
use crossbeam::Sender;

fn enumerate(path: impl AsRef<Path>, mut func: impl FnMut(DirEntry) -> ()) {
    for entry in WalkDir::new(path.as_ref()).follow_links(true).same_file_system(false) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                func(entry);
            }
        }
    }
}

fn add_file(sender: &Sender<File>, shared: &SharedData, stats: &Stats, file: File) {
    sender.send(file).unwrap();
    shared.total.total.fetch_add(1, Ordering::SeqCst);
    stats.total.fetch_add(1, Ordering::SeqCst);
}

pub fn enumerate_all_files(options: Options, sender: Sender<File>, shared: Arc<SharedData>) {
    let mut entry_func = |entry: DirEntry| {
        let shared = shared.as_ref();
        let path = entry.into_path();
        match path.file_name().map(|v| v.to_string_lossy().to_lowercase()) {
            p if p == Some("train.dat".into()) => add_file(
                &sender,
                shared,
                &shared.train_dat,
                File {
                    path,
                    kind: FileKind::TrainDat,
                },
            ),
            p if p == Some("train.xml".into()) => add_file(
                &sender,
                shared,
                &shared.train_xml,
                File {
                    path,
                    kind: FileKind::TrainXML,
                },
            ),
            p if p == Some("extensions.cfg".into()) => add_file(
                &sender,
                shared,
                &shared.extensions_cfg,
                File {
                    path,
                    kind: FileKind::ExtensionsCfg,
                },
            ),
            p if p == Some("panel.cfg".into()) => add_file(
                &sender,
                shared,
                &shared.panel_cfg,
                File {
                    path,
                    kind: FileKind::PanelCfg,
                },
            ),
            p if p == Some("panel2.cfg".into()) => add_file(
                &sender,
                shared,
                &shared.panel_cfg2,
                File {
                    path,
                    kind: FileKind::PanelCfg2,
                },
            ),
            p if p == Some("sound.cfg".into()) => add_file(
                &sender,
                shared,
                &shared.sound_cfg,
                File {
                    path,
                    kind: FileKind::SoundCfg,
                },
            ),
            p if p == Some("ats.cfg".into()) => add_file(
                &sender,
                shared,
                &shared.ats_cfg,
                File {
                    path,
                    kind: FileKind::AtsCfg,
                },
            ),
            _ => match path.extension().map(|v| v.to_string_lossy().to_lowercase()) {
                ext if ext == Some("b3d".into()) => add_file(
                    &sender,
                    shared,
                    &shared.model_b3d,
                    File {
                        path,
                        kind: FileKind::ModelB3d,
                    },
                ),
                ext if ext == Some("csv".into()) => add_file(
                    &sender,
                    shared,
                    &shared.model_csv,
                    File {
                        path,
                        kind: FileKind::ModelCsv,
                    },
                ),
                ext if ext == Some("animated".into()) => add_file(
                    &sender,
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

    enumerate(path, |entry| {
        let shared = shared.as_ref();
        let path = entry.into_path();
        match path.extension().map(|v| v.to_string_lossy().to_lowercase()) {
            ext if ext == Some("csv".into()) => add_file(
                &sender,
                shared,
                &shared.route_csv,
                File {
                    path,
                    kind: FileKind::RouteCsv,
                },
            ),
            ext if ext == Some("rw".into()) => add_file(
                &sender,
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
}
