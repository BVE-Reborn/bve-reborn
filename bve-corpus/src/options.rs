use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Clone)]
pub struct Options {
    /// Location of root of bve folder
    pub root_path: PathBuf,
}
