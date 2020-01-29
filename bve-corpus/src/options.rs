#![allow(clippy::option_unwrap_used)] // Internal to structopt

use std::path::PathBuf;
use structopt::StructOpt;

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
}
