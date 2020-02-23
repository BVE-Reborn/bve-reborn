use bve::filesystem::read_convert_utf8;
use bve::parse::animated::parse_animated_file;
use bve::parse::mesh::mesh_from_str;
use clap::arg_enum;
use std::path::{Path, PathBuf};
use std::time::Instant;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug, Clone)]
    enum FileType {
        B3D,
        CSV,
        Animated,
    }
}

#[derive(Debug, Clone, StructOpt)]
struct Options {
    #[structopt(possible_values = &FileType::variants(), case_insensitive = true)]
    file_type: FileType,

    /// show errors
    #[structopt(long)]
    errors: bool,

    /// file to load
    source_file: PathBuf,
}

fn parse_mesh_b3d_csv(file: impl AsRef<Path>, errors: bool, b3d: bool) {
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

    if errors {
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

fn parse_mesh_animated(file: impl AsRef<Path>, errors: bool) {
    let contents = read_convert_utf8(file).expect("Must be able to read file");

    let start = Instant::now();
    let (_parsed, warnings) = parse_animated_file(&contents);
    let duration = Instant::now() - start;

    println!("Duration: {:.4}", duration.as_secs_f32());

    if errors {
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
        FileType::B3D => parse_mesh_b3d_csv(&options.source_file, options.errors, true),
        FileType::CSV => parse_mesh_b3d_csv(&options.source_file, options.errors, false),
        FileType::Animated => parse_mesh_animated(&options.source_file, options.errors),
    }
}
