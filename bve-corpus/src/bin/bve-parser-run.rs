use bve::filesystem::read_convert_utf8;
use bve::parse::animated::parse_animated_file;
use bve::parse::ats_cfg::parse_ats_cfg;
use bve::parse::extensions_cfg::parse_extensions_cfg;
use bve::parse::mesh::mesh_from_str;
use bve::parse::panel1_cfg::parse_panel1_cfg;
use bve::parse::panel2_cfg::parse_panel2_cfg;
use bve::parse::sound_cfg::parse_sound_cfg;
use bve::parse::train_dat::parse_train_dat;
use clap::arg_enum;
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

fn parse_mesh_b3d_csv(file: impl AsRef<Path>, options: &Options, b3d: bool) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let file_type = if b3d {
        bve::parse::mesh::FileType::B3D
    } else {
        bve::parse::mesh::FileType::CSV
    };

    let start = Instant::now();
    let parsed = mesh_from_str(&contents, file_type);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        println!("{:#?}", &parsed);
    }

    if options.errors {
        println!("Warnings:");
        for e in &parsed.warnings {
            println!("\t{} {:?}", e.location.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
        println!("Errors:");
        for e in &parsed.errors {
            println!("\t{} {:?}", e.location.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Errors: {}", parsed.errors.len());
    }
}

fn parse_mesh_animated(file: impl AsRef<Path>, options: &Options) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let start = Instant::now();
    let (parsed, warnings) = parse_animated_file(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        println!("{:#?}", parsed);
    }

    if options.errors {
        println!("Warnings:");
        for e in &warnings {
            println!("\t{} {:?}", e.span.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Warnings: {}", warnings.len());
    }
}

fn parse_config_train_dat(file: impl AsRef<Path>, options: &Options) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let start = Instant::now();
    let (parsed, warnings) = parse_train_dat(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        println!("{:#?}", parsed);
    }

    if options.errors {
        println!("Warnings:");
        for e in &warnings {
            println!("\t{} {:?}", e.span.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Warnings: {}", warnings.len());
    }
}

fn parse_config_extensions_cfg(file: impl AsRef<Path>, options: &Options) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let start = Instant::now();
    let (parsed, warnings) = parse_extensions_cfg(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        println!("{:#?}", parsed);
    }

    if options.errors {
        println!("Warnings:");
        for e in &warnings {
            println!("\t{} {:?}", e.span.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Warnings: {}", warnings.len());
    }
}

fn parse_config_ats_cfg(file: impl AsRef<Path>, options: &Options) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let start = Instant::now();
    let (parsed, warnings) = parse_ats_cfg(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        println!("{:#?}", parsed);
    }

    if options.errors {
        println!("Warnings:");
        for e in &warnings {
            println!("\t{} {:?}", e.span.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Warnings: {}", warnings.len());
    }
}

fn parse_config_panel1_cfg(file: impl AsRef<Path>, options: &Options) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let start = Instant::now();
    let (parsed, warnings) = parse_panel1_cfg(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        println!("{:#?}", parsed);
    }

    if options.errors {
        println!("Warnings:");
        for e in &warnings {
            println!("\t{} {:?}", e.span.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Warnings: {}", warnings.len());
    }
}

fn parse_config_panel2_cfg(file: impl AsRef<Path>, options: &Options) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let start = Instant::now();
    let (parsed, warnings) = parse_panel2_cfg(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        println!("{:#?}", parsed);
    }

    if options.errors {
        println!("Warnings:");
        for e in &warnings {
            println!("\t{} {:?}", e.span.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Warnings: {}", warnings.len());
    }
}

fn parse_config_sound_cfg(file: impl AsRef<Path>, options: &Options) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let start = Instant::now();
    let (parsed, warnings) = parse_sound_cfg(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if options.print_result {
        println!("{:#?}", parsed);
    }

    if options.errors {
        println!("Warnings:");
        for e in &warnings {
            println!("\t{} {:?}", e.span.line.map(|v| v as i64).unwrap_or(-1), e.kind)
        }
    } else {
        println!("Warnings: {}", warnings.len());
    }
}

fn main() {
    let options: Options = Options::from_args();

    match options.file_type {
        FileType::AtsCfg => parse_config_ats_cfg(&options.source_file, &options),
        FileType::B3D => parse_mesh_b3d_csv(&options.source_file, &options, true),
        FileType::CSV => parse_mesh_b3d_csv(&options.source_file, &options, false),
        FileType::Animated => parse_mesh_animated(&options.source_file, &options),
        FileType::TrainDat => parse_config_train_dat(&options.source_file, &options),
        FileType::ExtensionsCfg => parse_config_extensions_cfg(&options.source_file, &options),
        FileType::PanelCfg => parse_config_panel1_cfg(&options.source_file, &options),
        FileType::Panel2Cfg => parse_config_panel2_cfg(&options.source_file, &options),
        FileType::SoundCfg => parse_config_sound_cfg(&options.source_file, &options),
    }
}
