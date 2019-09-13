use std::process::Command;

fn main() {
    let mut child = Command::new("cargo").args(&["build", "--release"]).spawn().unwrap();
    assert!(child.wait().unwrap().success());
}
