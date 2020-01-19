// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]
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
#![allow(clippy::option_expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::result_expect_used)]
#![allow(clippy::result_unwrap_used)] // Doesn't play nice with structopt
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]
#![feature(core_panic)] // CLion is having a fit about panic
#[allow(unused)] // CLion is having a fit about panic
use core::panicking::panic;

use cbindgen::Language;
use std::fs::read_to_string;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    /// Passthrough option to cargo for the color option
    #[structopt(short, long)]
    color: Option<String>,

    /// Build bve in debug mode
    #[structopt(long)]
    debug: bool,

    /// Don't build bve
    #[structopt(long)]
    no_build: bool,

    /// Don't run cbindgen
    #[structopt(long)]
    no_bindgen: bool,
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

    let mut child = Command::new("cargo")
        .args(&args)
        .spawn()
        .expect("Unable to spawn cargo.");
    assert!(child.wait().expect("Unable to wait for cargo.").success());
}

fn handle_cbindgen_error(err: cbindgen::Error, options: &Options) {
    match &err {
        cbindgen::Error::CargoExpand(s, err) => {
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
        _ => {}
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
            Err(err) => handle_cbindgen_error(err, options),
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
        cbindgen::Builder::new()
            .with_crate("bve-native")
            .with_config(config)
            .generate()
            .unwrap()
            .write_to_file("bve-native/include/bve.hpp");
    }
}

fn main() {
    let options: Options = Options::from_args();

    if !options.no_build {
        build(&options);
    }

    if !options.no_bindgen {
        generate_c_bindings(&options)
    }
}
