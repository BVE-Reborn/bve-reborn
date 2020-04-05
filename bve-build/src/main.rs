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
#![allow(clippy::else_if_without_else)]
#![allow(clippy::exit)]
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

use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{metadata, read_dir},
    path::Path,
    process::{exit, Command},
};

#[allow(clippy::struct_excessive_bools)]
struct Options {
    /// Passthrough option to cargo for the color option
    color: Option<String>,

    /// Help
    help: bool,

    /// Build bve in debug mode
    debug: bool,

    /// Build bve
    build: bool,

    /// Run cbindgen
    cbindgen: bool,

    /// Build shaders
    shaderc: bool,

    /// Build `bve` crate
    core: bool,

    /// Build `bve-client` crate
    client: bool,

    /// Build `bve-native` crate
    native: bool,
}

fn out_of_date(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> bool {
    if !dst.as_ref().exists() {
        return true;
    }

    let src_time = metadata(src.as_ref())
        .expect("Source must have metadata")
        .modified()
        .expect("Source must have modified time");
    let dst_time = metadata(dst.as_ref())
        .expect("Destination must have metadata")
        .modified()
        .expect("Destination must have modified time");

    src_time > dst_time
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

fn generate_c_bindings() {
    println!("Generating C Bindings... (may take a while)");
    let mut c = Command::new("cbindgen")
        .args(&["--crate", "bve-native", "-o", "include/bve.h", "-c", "cbindgen-c.toml"])
        .current_dir("bve-native")
        .spawn()
        .unwrap();
    if !c.wait().unwrap().success() {
        println!("cbindgen failed");
    };
    println!("Generating C++ Bindings... (may take a while)");
    let mut cpp = Command::new("cbindgen")
        .args(&[
            "--crate",
            "bve-native",
            "-o",
            "include/bve.hpp",
            "-c",
            "cbindgen-cpp.toml",
        ])
        .current_dir("bve-native")
        .spawn()
        .unwrap();
    if !cpp.wait().unwrap().success() {
        println!("cbindgen failed");
    };
}

fn build_shaders() {
    for content in read_dir("bve-render/shaders").unwrap() {
        let entry = content.expect("IO error");
        if !entry.file_type().unwrap().is_dir()
            && entry.path().extension().map(OsStr::to_string_lossy) == Some(Cow::Borrowed("glsl"))
        {
            let name = entry.file_name().to_string_lossy().to_string();
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

            {
                let spirv_name = name.replace(".glsl", ".spv");
                let out_path = entry.path().parent().expect("Must have parent").join(&spirv_name);

                if out_of_date(entry.path(), &out_path) {
                    println!("Compiling {} to {}", name, spirv_name);

                    let mut child = Command::new("glslc")
                        .args(&[
                            "-x",
                            "glsl",
                            "-g",
                            stage,
                            &format!("{}", entry.path().display()),
                            "-o",
                            &format!("{}", out_path.display()),
                        ])
                        .spawn()
                        .expect(
                            "Unable to find glslc in PATH. glslc must be installed. See https://github.com/google/shaderc",
                        );

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

            {
                let spirv_name_opt = name.replace(".glsl", ".spv.opt");
                let out_path_opt = entry.path().parent().expect("Must have parent").join(&spirv_name_opt);

                if out_of_date(entry.path(), &out_path_opt) {
                    println!("Compiling {} to {}", name, spirv_name_opt);

                    let mut child = Command::new("glslc")
                        .args(&[
                            "-x",
                            "glsl",
                            "-O",
                            stage,
                            &format!("{}", entry.path().display()),
                            "-o",
                            &format!("{}", out_path_opt.display()),
                        ])
                        .spawn()
                        .expect(
                            "Unable to find glslc in PATH. glslc must be installed. See https://github.com/google/shaderc",
                        );

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
    }
}

const HELP_MESSAGE: &str = r#"Usage: cargo run --bin bve-build -- [args...]
BVE-Reborn build tool.

General:
  -h, --help       Display this help message
  --color=[value]  Pass --color=[value] to all cargo calls
  
Tasks:
  --build     Build bve-reborn
  --cbindgen  Build bve-native C and C++ headers (requires cbindgen)
  --shaderc   Build spirv shaders (requires glslc [from shaderc])

Build Options:
  --debug   Build bve in debug mode
  
Libraries:
  If none of these are specified, builds everything
  --core    Build bve-core
  --client  Build bve-client
  --native  Build bve-native
"#;

fn main() {
    let mut args = pico_args::Arguments::from_env();

    let mut options: Options = Options {
        color: args.opt_value_from_str("--color").unwrap(),
        help: args.contains(["-h", "--help"]),
        debug: args.contains("--debug"),
        build: args.contains("--build"),
        cbindgen: args.contains("--cbindgen"),
        shaderc: args.contains("--shaderc"),
        core: args.contains("--core"),
        client: args.contains("--client"),
        native: args.contains("--native"),
    };

    if let Err(pico_args::Error::UnusedArgsLeft(args)) = args.finish() {
        println!("Unrecognized arguments: {}", args.join(", "));
        options.help = true;
    }

    if options.help {
        println!("{}", HELP_MESSAGE);
        exit(1);
    }

    let all = !(options.cbindgen || options.build || options.shaderc);

    if options.build || all {
        build(&options);
    }

    if options.cbindgen || all {
        generate_c_bindings();
    }

    if options.shaderc || all {
        build_shaders();
    }
}
