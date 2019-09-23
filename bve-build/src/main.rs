// Rust warnings
#![warn(unused)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
#![allow(clippy::cast_sign_loss)] // Annoying
#![allow(clippy::cast_precision_loss)] // Annoying
#![allow(clippy::cast_possible_truncation)] // Annoying
#![allow(clippy::cognitive_complexity)] // This is dumb
#![allow(clippy::multiple_crate_versions)] // Dependencies are hard
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
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::wildcard_enum_match_arm)]

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
}
