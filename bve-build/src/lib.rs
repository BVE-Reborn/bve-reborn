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
#![allow(clippy::expect_used)]
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
#![allow(clippy::panic)]
#![allow(clippy::print_stdout)] // This is a build script, not a fancy app
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::string_add)]
#![allow(clippy::string_add_assign)]
#![allow(clippy::unreachable)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::wildcard_enum_match_arm)]

use crate::shaders::{ShaderCombination, ShaderType, SingleDefine};
use itertools::Itertools;
use std::{
    fs::{create_dir_all, metadata, read_dir, read_to_string, remove_dir_all},
    path::{Path, PathBuf},
    process::{exit, Command},
};

mod shaders;

#[allow(clippy::struct_excessive_bools)]
pub struct Options {
    /// Passthrough option to cargo for the color option
    pub color: Option<String>,

    /// Help
    pub help: bool,

    /// Build bve in debug mode
    pub debug: bool,

    /// Build bve
    pub build: bool,

    /// Run cbindgen
    pub cbindgen: bool,

    /// Build shaders
    pub shaderc: bool,

    /// Build `bve` crate
    pub core: bool,

    /// Build `bve-client` crate
    pub client: bool,

    /// Build `bve-native` crate
    pub native: bool,
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

fn headers_out_of_date(bve_render: &Path, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> bool {
    for element in read_dir(&bve_render.join("shaders/include")).expect("could not find include dir") {
        let element = element.expect("has no element");
        if out_of_date(element.path(), src.as_ref()) && out_of_date(element.path(), dst.as_ref()) {
            return true;
        }
    }
    false
}

pub fn build(options: &Options) {
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

pub fn generate_c_bindings() {
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

struct ShaderName {
    debug: PathBuf,
    release: PathBuf,
}

fn mangle_shader_name(out_dir: &Path, combination: &ShaderCombination<'_>) -> ShaderName {
    let mut input = combination.name.to_string();
    if !combination.defines.is_empty() {
        input.push('_');
        input.push_str(
            &combination
                .defines
                .iter()
                .map(|define| match define {
                    shaders::SingleDefine::Defined(key, value) => format!("{}_{}", key, value),
                    shaders::SingleDefine::Undefined(key) => format!("U{}", key),
                })
                .join("_"),
        );
    }
    match combination.ty {
        ShaderType::Vertex => {
            input.push_str(".vs.spv");
        }
        ShaderType::Fragment => {
            input.push_str(".fs.spv");
        }
        ShaderType::Compute => {
            input.push_str(".cs.spv");
        }
    };
    ShaderName {
        debug: out_dir.join(format!("shaders/spirv-debug/{}", input)),
        release: out_dir.join(format!("shaders/spirv/{}", input)),
    }
}

pub fn build_shaders(out_dir: &Path, release: bool) {
    let bve_render = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../bve-render"));
    if Path::new(&out_dir.join("shaders/spirv-debug")).exists()
        && out_of_date(
            &bve_render.join("shaders/compile"),
            &out_dir.join("shaders/spirv-debug/"),
        )
        && out_of_date(
            &bve_render.join("shaders/include"),
            &out_dir.join("shaders/spirv-debug/"),
        )
    {
        remove_dir_all(&out_dir.join("shaders/spirv-debug")).expect("Could not remove directory");
        println!("Removing out of date spirv-debug directory")
    }
    create_dir_all(&out_dir.join("shaders/spirv-debug")).expect("Could not create spirv-debug directory");

    if Path::new(&out_dir.join("shaders/spirv")).exists()
        && out_of_date(&bve_render.join("shaders/compile"), &out_dir.join("shaders/spirv/"))
        && out_of_date(&bve_render.join("shaders/include"), &out_dir.join("shaders/spirv/"))
    {
        remove_dir_all(&out_dir.join("shaders/spirv")).expect("Could not remove directory");
        println!("Removing out of date spirv directory")
    }
    create_dir_all(&out_dir.join("shaders/spirv")).expect("Could not create spirv directory");

    for combination in
        shaders::parse_shader_compile_file(&read_to_string(bve_render.join("shaders/compile")).unwrap()).unwrap()
    {
        let mut source_name = bve_render.join(format!("shaders/{}", &combination.name));
        let spirv_name = mangle_shader_name(out_dir, &combination);
        let stage = match combination.ty {
            ShaderType::Vertex => {
                source_name.set_extension("vs.glsl");
                "-fshader-stage=vertex"
            }
            ShaderType::Fragment => {
                source_name.set_extension("fs.glsl");
                "-fshader-stage=fragment"
            }
            ShaderType::Compute => {
                source_name.set_extension("cs.glsl");
                "-fshader-stage=compute"
            }
        };

        let define_flags = combination
            .defines
            .into_iter()
            .filter_map(|define| {
                if let SingleDefine::Defined(key, value) = define {
                    Some(format!("-D{}={}", key, value))
                } else {
                    None
                }
            })
            .chain(
                [
                    String::from("-I"),
                    String::from(bve_render.join("shaders/include").to_string_lossy()),
                ]
                .iter()
                .cloned(),
            )
            .collect_vec();

        {
            if (out_of_date(&source_name, &spirv_name.debug)
                || headers_out_of_date(bve_render, &source_name, &spirv_name.debug))
                && !release
            {
                println!("Compiling {} to {}", source_name.display(), spirv_name.debug.display());

                let mut flags = define_flags.clone();
                flags.extend(
                    [
                        "-x",
                        "glsl",
                        "-g",
                        "-O",
                        stage,
                        &source_name.to_string_lossy(),
                        "-o",
                        &spirv_name.debug.to_string_lossy(),
                    ]
                    .iter()
                    .map(|&s| s.to_string()),
                );
                let mut child = Command::new("glslc").args(&flags).spawn().expect(
                    "Unable to find glslc in PATH. glslc must be installed. See https://github.com/google/shaderc",
                );

                let result = child.wait().expect("Unable to wait for child");
                if !result.success() {
                    println!(
                        "glslc failed on file {} with error code {}",
                        source_name.display(),
                        result.code().expect("Unable to get error code")
                    );
                    exit(1);
                }
            } else if !release {
                println!(
                    "Ignoring {}. {} already up to date.",
                    source_name.display(),
                    spirv_name.debug.display()
                );
            }
        }

        {
            if (out_of_date(&source_name, &spirv_name.release)
                || headers_out_of_date(bve_render, &source_name, &spirv_name.release))
                && release
            {
                println!(
                    "Compiling {} to {}",
                    source_name.display(),
                    spirv_name.release.display()
                );

                let mut flags = define_flags.clone();
                flags.extend(
                    [
                        "-x",
                        "glsl",
                        "-O",
                        stage,
                        &source_name.to_string_lossy(),
                        "-o",
                        &spirv_name.release.to_string_lossy(),
                    ]
                    .iter()
                    .map(|&s| s.to_string()),
                );
                let mut child = Command::new("glslc").args(&flags).spawn().expect(
                    "Unable to find glslc in PATH. glslc must be installed. See https://github.com/google/shaderc",
                );

                let result = child.wait().expect("Unable to wait for child");
                if !result.success() {
                    println!(
                        "glslc failed on file {} with error code {}",
                        source_name.display(),
                        result.code().expect("Unable to get error code")
                    );
                    exit(1);
                }
            } else if release {
                println!(
                    "Ignoring {}. {} already up to date.",
                    source_name.display(),
                    spirv_name.debug.display()
                );
            }
        }
    }
}
