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
