use bve::{
    filesystem::read_convert_utf8,
    parse::{
        animated::ParsedAnimatedObject,
        ats_cfg::ParsedAtsConfig,
        extensions_cfg::ParsedExtensionsCfg,
        mesh::{ParsedStaticObjectB3D, ParsedStaticObjectCSV},
        panel1_cfg::ParsedPanel1Cfg,
        panel2_cfg::ParsedPanel2Cfg,
        sound_cfg::ParsedSoundCfg,
        train_dat::ParsedTrainDat,
        FileParser, ParserResult, PrettyPrintResult, UserError,
    },
};
use log::{error, info, warn};
use std::{
    convert::TryFrom,
    io::stdout,
    path::{Path, PathBuf},
    process::exit,
    str::FromStr,
    time::Instant,
};

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
            "sound" | "sound.cfg" => Self::Panel2Cfg,
            _ => return Err(format!("Invalid File Type: {}", lower)),
        })
    }
}

#[derive(Clone)]
pub struct Arguments {
    pub help: bool,
    pub source_file: PathBuf,
    pub errors: bool,
    pub print_result: bool,
    pub file_type: FileType,

    pub log_output: Option<PathBuf>,
    pub quiet: bool,
    pub debug: bool,
    pub trace: bool,
}

const HELP_MESSAGE: &str = r#"cargo run --bin bve-parser-run -- [options] <type> <path>
BVE-Reborn parser runner -- runs a single file with specific aprser

General Options:
  <type>     File type to test. Options:
               ats[.cfg]
               b3d
               csv-mesh
               anim[ated]
               train[.dat]
               ext[ensions.cfg]
               panel[1][.cfg]
               panel2[.cfg]
               sound[.cfg]
  <path>     Path to file to test
  -h,--help  Print this message
  
Printing Options:
  -e,--errors  Print all warnings/errors
  -p,--print   Print parser output
                 
Logging Options:
  --log        Send all messages to a file. Errors and warnings
                 will also be sent to stderr as normal.
  -q,--quiet   Disable info level log messages
  -v,--debug   Enable debug trace level log messages
  -vv,--trace  Enable trace level log messages
"#;

impl Arguments {
    #[allow(clippy::redundant_closure)] // PathBuf::try_from doesn't work
    pub fn create(mut args: pico_args::Arguments) -> Result<Self, String> {
        let o = Self {
            help: args.contains(["-h", "--help"]),
            errors: args.contains(["-e", "--errors"]),
            print_result: args.contains(["-p", "--print"]),

            log_output: args
                .opt_value_from_os_str("--log", |os| PathBuf::try_from(os))
                .map_err(|e| e.to_string())?,
            quiet: args.contains(["-q", "--quiet"]),
            debug: args.contains(["-v", "--debug"]),
            trace: args.contains(["-vv", "--trace"]),

            file_type: args
                .free_from_str()
                .map_err(|e| e.to_string())?
                .ok_or_else(|| String::from("Missing file type"))?,
            source_file: args
                .free_from_os_str(|os| PathBuf::try_from(os))
                .map_err(|e| e.to_string())?
                .ok_or_else(|| String::from("No path provided"))?,
        };

        args.finish().map_err(|e| e.to_string())?;

        Ok(o)
    }

    // Pretend to be structopt lmao
    #[must_use]
    pub fn from_args() -> Self {
        let o = Self::create(pico_args::Arguments::from_env());

        match o {
            Ok(Arguments { help: true, .. }) => {
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

fn parse_file<T: FileParser>(source: &Path, options: &Arguments) {
    let contents = read_convert_utf8(source).expect("Must be able to read file");

    let start = Instant::now();
    let ParserResult {
        output,
        warnings,
        errors,
    } = T::parse_from(&contents);
    let duration = Instant::now() - start;

    info!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        output
            .fmt(0, &mut stdout().lock())
            .expect("Must be able to write to stdout");
    }

    if options.errors && !warnings.is_empty() {
        info!("Warnings:");
        for w in warnings {
            let w = w.to_data();
            warn!("\t{} {:?}", w.line, w.description_english);
        }
    } else {
        info!("Warnings: {}", warnings.len());
    }
    if options.errors && !errors.is_empty() {
        info!("Errors:");
        for e in errors {
            let e = e.to_data();
            error!("\t{} {:?}", e.line, e.description_english);
        }
    } else {
        info!("Errors: {}", errors.len());
    }
}

fn main() {
    let options: Arguments = Arguments::from_args();

    bve::log::enable_logger(&options.log_output, options.quiet, options.debug, options.trace);

    match options.file_type {
        FileType::AtsCfg => parse_file::<ParsedAtsConfig>(&options.source_file, &options),
        FileType::B3D => parse_file::<ParsedStaticObjectB3D>(&options.source_file, &options),
        FileType::CSV => parse_file::<ParsedStaticObjectCSV>(&options.source_file, &options),
        FileType::Animated => parse_file::<ParsedAnimatedObject>(&options.source_file, &options),
        FileType::TrainDat => parse_file::<ParsedTrainDat>(&options.source_file, &options),
        FileType::ExtensionsCfg => parse_file::<ParsedExtensionsCfg>(&options.source_file, &options),
        FileType::PanelCfg => parse_file::<ParsedPanel1Cfg>(&options.source_file, &options),
        FileType::Panel2Cfg => parse_file::<ParsedPanel2Cfg>(&options.source_file, &options),
        FileType::SoundCfg => parse_file::<ParsedSoundCfg>(&options.source_file, &options),
    }
}
