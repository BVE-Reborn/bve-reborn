use pico_args::Arguments;
use std::{convert::TryFrom, path::PathBuf, process::exit, str::FromStr};

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
    SoundCfg,
}

impl FromStr for FileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        Ok(match lower.as_str() {
            "ats" | "ats.cfg" => Self::AtsCfg,
            "b3d" => Self::B3D,
            "csv-mesh" => Self::CSV,
            "anim" | "animated" => Self::Animated,
            "train" | "train.dat" => Self::TrainDat,
            "ext" | "extensions.cfg" => Self::ExtensionsCfg,
            "panel" | "panel1" | "panel1.cfg" | "panel.cfg" => Self::PanelCfg,
            "panel2" | "panel2.cfg" => Self::Panel2Cfg,
            "sound" | "sound.cfg" => Self::SoundCfg,
            _ => return Err(format!("Invalid File Type: {}", lower)),
        })
    }
}

#[derive(Clone)]
pub struct Options {
    pub help: bool,
    /// Location of root of bve folder
    pub path: PathBuf,
    /// Location of result file
    pub output: Option<PathBuf>,
    /// Job Count
    pub jobs: Option<usize>,
    /// File to allow
    pub file_types: Option<FileType>,
}

const HELP_MESSAGE: &str = r#"cargo run --bin bve-corpus -- [options] <path>
BVE-Reborn corpus tester -- tests bve parsers against an entire OpenBVE data folder

General Options:
  <path>       Path to OpenBVE folder
  -h,--help    Print this message
  -j,--jobs    Worker threads (there will be 1 more filesystem scanning thread)
  -o,--output  Output json report to file
  -f,--file    File type to test. If not added, will test all files. Options:
                 ats[.cfg]
                 b3d
                 csv-mesh
                 anim[ated]
                 train[.dat]
                 ext[ensions.cfg]
                 panel[1][.cfg]
                 panel2[.cfg]
                 sound[.cfg]
"#;

impl Options {
    #[allow(clippy::redundant_closure)] // PathBuf::try_from doesn't work
    fn create(mut args: Arguments) -> Result<Self, String> {
        let o = Self {
            help: args.contains(["-h", "--help"]),
            path: args
                .free_from_os_str(|os| PathBuf::try_from(os))
                .map_err(|e| e.to_string())?
                .ok_or_else(|| String::from("No path provided"))?,
            output: args
                .opt_value_from_os_str(["-o", "--output"], |v| PathBuf::try_from(v))
                .map_err(|e| e.to_string())?,
            jobs: args.opt_value_from_str(["-j", "--jobs"]).map_err(|e| e.to_string())?,
            file_types: args.opt_value_from_str(["-f", "--file"]).map_err(|e| e.to_string())?,
        };

        args.finish().map_err(|e| e.to_string())?;

        Ok(o)
    }

    // Pretend to be structopt lmao
    #[must_use]
    pub fn from_args() -> Self {
        let o = Self::create(Arguments::from_env());

        match o {
            Ok(Self { help: true, .. }) => {
                println!("{}", HELP_MESSAGE);
                exit(0);
            }
            Err(e) => {
                println!("Error parsing args: {}\n{}", e, HELP_MESSAGE);
                exit(1);
            }
            Ok(o) => o,
        }
    }
}
