// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]
// Rustdoc Warnings
#![deny(intra_doc_link_resolution_failure)]
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
// Annoying regular clippy warnings
#![allow(clippy::cast_sign_loss)] // Annoying
#![allow(clippy::cast_precision_loss)] // Annoying
#![allow(clippy::cast_possible_truncation)] // Annoying
#![allow(clippy::cognitive_complexity)] // This is dumb
#![allow(clippy::too_many_lines)] // This is also dumb
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::as_conversions)]
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::let_underscore_must_use)]
#![allow(clippy::match_bool)] // prettier
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)] // Cargo deny's job
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::option_expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::print_stdout)] // This is a build script, not a fancy app
#![allow(clippy::result_expect_used)]
#![allow(clippy::result_unwrap_used)] // Doesn't play nice with structopt
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]
// CLion is having a fit about panic not existing
#![feature(core_panic)]
#![allow(unused_imports)]
use core::panicking::panic;

use cbindgen::Language;
use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{read_to_string, FileType},
    process::{exit, Command},
};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt)]
struct Options {
    /// Passthrough option to cargo for the color option
    #[structopt(short, long)]
    color: Option<String>,

    /// Build bve in debug mode
    #[structopt(long)]
    debug: bool,

    /// Build bve
    #[structopt(long)]
    build: bool,

    /// Run cbindgen
    #[structopt(long)]
    bindgen: bool,

    /// Build shaders
    #[structopt(long)]
    shaderc: bool,

    /// Build `bve` crate
    #[structopt(long)]
    core: bool,

    /// Build `bve-client` crate
    #[structopt(long)]
    client: bool,

    /// Build `bve-native` crate
    #[structopt(long)]
    native: bool,
}

fn clean() {
    let mut child = Command::new("cargo")
        .arg("clean")
        .spawn()
        .expect("Unable to spawn cargo.");
    assert!(child.wait().expect("Unable to wait for cargo.").success());
}

fn build(options: &Options) {
    let mut args = if options.debug {
        vec![String::from("build")]
    } else {
        vec![String::from("build"), String::from("--release")]
    };

    options.color.iter().for_each(|s| args.push(format!("--color={}", s)));

    // what is DRY?
    if options.core {
        args.push(String::from("-p"));
        args.push(String::from("bve"));
    }
    if options.client {
        args.push(String::from("-p"));
        args.push(String::from("bve-client"));
    }
    if options.native {
        args.push(String::from("-p"));
        args.push(String::from("bve-native"));
    }
    if !(options.core || options.client || options.native) {
        args.push(String::from("-p"));
        args.push(String::from("bve"));
        args.push(String::from("-p"));
        args.push(String::from("bve-client"));
        args.push(String::from("-p"));
        args.push(String::from("bve-native"));
    }

    let mut child = Command::new("cargo")
        .args(&args)
        .spawn()
        .expect("Unable to spawn cargo.");
    assert!(child.wait().expect("Unable to wait for cargo.").success());
}

fn handle_cbindgen_error(err: &cbindgen::Error, options: &Options) {
    if let cbindgen::Error::CargoExpand(s, err) = &err {
        // Bug in cbindgen/rustc
        // https://github.com/eqrion/cbindgen/issues/457
        // https://github.com/rust-lang/rust/issues/68333
        // Fixed by cleaning the build cache and rerunning
        if err.to_string().contains("Finished") && s == "bve-native" {
            println!("Dealing with cbindgen bug; clearing cache and regenerating");
            clean();
            generate_c_bindings(options);
            return;
        }
    }
    panic!("cbindgen error: {}", err)
}

fn generate_c_bindings(options: &Options) {
    let config = cbindgen::Config::from_file("bve-native/cbindgen.toml").unwrap();

    {
        // C
        let mut config = config.clone();
        config.language = Language::C;
        *config.header.as_mut().expect("bve-native/cbindgen.toml needs a header") +=
            "/* C API for BVE-Reborn high performance libraries. */";
        let result = cbindgen::Builder::new()
            .with_crate("bve-native")
            .with_config(config)
            .generate();
        match result {
            Ok(bindings) => {
                bindings.write_to_file("bve-native/include/bve.h");
            }
            Err(err) => handle_cbindgen_error(&err, options),
        }
    }
    {
        // C++
        let mut config = config;
        config.language = Language::Cxx;
        config.export.prefix = None;
        *config.header.as_mut().expect("bve-native/cbindgen.toml needs a header") +=
            "/* C++ API for BVE-Reborn high performance libraries. */";
        config.trailer = Some(read_to_string("bve-native/include/bve_cpp.hpp").unwrap());
        let result = cbindgen::Builder::new()
            .with_crate("bve-native")
            .with_config(config)
            .generate();
        match result {
            Ok(bindings) => {
                bindings.write_to_file("bve-native/include/bve.hpp");
            }
            Err(err) => handle_cbindgen_error(&err, options),
        }
    }
}

fn build_shaders() {
    for content in WalkDir::new("bve-render/shaders") {
        let entry = content.expect("IO error");
        if !entry.file_type().is_dir()
            && entry.path().extension().map(OsStr::to_string_lossy) == Some(Cow::Borrowed("glsl"))
        {
            let name = entry.file_name().to_string_lossy();
            let stage = if name.contains(".vs") {
                "-fshader-stage=vert"
            } else if name.contains(".gs") {
                "-fshader-stage=geom"
            } else if name.contains(".fs") {
                "-fshader-stage=frag"
            } else if name.contains(".cs") {
                "-fshader-stage=comp"
            } else {
                break;
            };

            let spirv_name = name.replace(".glsl", ".spv");
            let out_path = entry.path().parent().expect("Must have parent").join(&spirv_name);

            println!("Compiling {} to {}", name, spirv_name);

            let mut child = Command::new("glslc")
                .args(&[
                    "-x",
                    "glsl",
                    stage,
                    "-O",
                    &format!("{}", entry.path().display()),
                    "-o",
                    &format!("{}", out_path.display()),
                ])
                .spawn()
                .expect("Unable to find glslc in PATH. glslc must be installed. See https://github.com/google/shaderc");

            let result = child.wait().expect("Unable to wait for child");
            if !result.success() {
                println!(
                    "glslc failed on file {} with error code {}",
                    name,
                    result.code().expect("Unable to get error code")
                );
                exit(1);
            }
        }
    }
}

fn main() {
    let options: Options = Options::from_args();

    let all = !(options.bindgen || options.build || options.shaderc);

    if options.build || all {
        build(&options);
    }

    if options.bindgen || all {
        generate_c_bindings(&options);
    }

    if options.shaderc || all {
        build_shaders();
    }
}
