#![allow(clippy::option_unwrap_used)] // Internal to structopt

use clap::arg_enum;
use std::path::PathBuf;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum FileType {
        AtsCfg,
        B3D,
        CSV,
        Animated,
        TrainDat,
        ExtensionsCfg,
        PanelCfg,
        Panel2Cfg,
    }
}

#[derive(StructOpt, Clone)]
pub struct Options {
    /// Location of root of bve folder
    pub root_path: PathBuf,
    /// Location of result file
    #[structopt(short, long)]
    pub output: Option<PathBuf>,
    /// Job Count
    #[structopt(short, long)]
    pub jobs: Option<usize>,
    /// File to allow
    #[structopt(short, long, possible_values = &FileType::variants(), case_insensitive = true)]
    pub file: Option<FileType>,
}
