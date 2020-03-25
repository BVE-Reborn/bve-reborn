use bve::filesystem::read_convert_utf8;
use bve::parse::animated::ParsedAnimatedObject;
use bve::parse::ats_cfg::ParsedAtsConfig;
use bve::parse::extensions_cfg::ParsedExtensionsCfg;
use bve::parse::mesh::{ParsedStaticObjectB3D, ParsedStaticObjectCSV};
use bve::parse::panel1_cfg::ParsedPanel1Cfg;
use bve::parse::panel2_cfg::ParsedPanel2Cfg;
use bve::parse::sound_cfg::ParsedSoundCfg;
use bve::parse::train_dat::ParsedTrainDat;
use bve::parse::{FileParser, ParserResult, PrettyPrintResult};
use clap::arg_enum;
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::time::Instant;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug, Clone)]
    enum FileType {
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
}

#[derive(Debug, Clone, StructOpt)]
struct Options {
    #[structopt(possible_values = &FileType::variants(), case_insensitive = true)]
    file_type: FileType,

    /// show errors
    #[structopt(long)]
    errors: bool,

    /// show result
    #[structopt(short, long = "print")]
    print_result: bool,

    /// file to load
    source_file: PathBuf,
}

fn parse_file<T: FileParser>(source: &Path, options: &Options) {
    let contents = read_convert_utf8(source).expect("Must be able to read file");

    let start = Instant::now();
    let ParserResult {
        output,
        warnings,
        errors,
    } = T::parse_from(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        output
            .fmt(0, &mut stdout().lock())
            .expect("Must be able to write to stdout");
    }

    if options.errors {
        println!("Warnings:");
        for _e in warnings {
            // println!("\t{} {:?}", e.location.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
        println!("Errors:");
        for _e in errors {
            // println!("\t{} {:?}", e.location.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Warnings: {}", warnings.len());
        println!("Errors: {}", errors.len());
    }
}

fn main() {
    let options: Options = Options::from_args();

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
