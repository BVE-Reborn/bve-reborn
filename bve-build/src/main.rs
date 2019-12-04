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
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::result_unwrap_used)] // Doesn't play nice with structopt
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::wildcard_enum_match_arm)]

use cbindgen::Language;
use std::fs::read_to_string;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    color: Option<String>,
}

fn main() {
    let parsed: Options = Options::from_args();

    let mut args = vec![String::from("build"), String::from("--release")];
    parsed.color.iter().for_each(|s| args.push(format!("--color={}", s)));

    let mut child = Command::new("cargo")
        .args(&args)
        .spawn()
        .expect("Unable to spawn cargo.");
    assert!(child.wait().expect("Unable to wait for cargo.").success());

    let config = cbindgen::Config::from_file("bve-native/cbindgen.toml").unwrap();

    {
        // C
        let mut config = config.clone();
        config.language = Language::C;
        *config.header.as_mut().expect("bve-native/cbindgen.toml needs a header") +=
            "/* C API for BVE-Reborn high performance libraries. */";
        cbindgen::Builder::new()
            .with_crate("bve-native")
            .with_config(config)
            .generate()
            .unwrap()
            .write_to_file("bve-native/include/bve.h");
    }
    {
        // C++
        let mut config = config;
        config.language = Language::Cxx;
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
