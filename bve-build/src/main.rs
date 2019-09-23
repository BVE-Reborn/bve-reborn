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
#![allow(clippy::cognitive_complexity)] // This is dumb
#![allow(clippy::multiple_crate_versions)] // Dependencies are hard
// Clippy Restrictions
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::get_unwrap)]
#![warn(clippy::mem_forget)]
#![warn(clippy::multiple_inherent_impl)]
#![warn(clippy::option_unwrap_used)]
#![warn(clippy::print_stdout)]
#![warn(clippy::result_unwrap_used)]
#![warn(clippy::unimplemented)]
#![warn(clippy::wildcard_enum_match_arm)]
#![warn(clippy::wrong_pub_self_convention)]

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

    let mut child = Command::new("cargo").args(&args).spawn().unwrap();
    assert!(child.wait().unwrap().success());
}
