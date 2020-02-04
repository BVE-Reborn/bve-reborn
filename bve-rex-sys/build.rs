use std::path::PathBuf;
use walkdir::{DirEntry, Error};

fn match_c_files(value: Result<DirEntry, Error>) -> Option<PathBuf> {
    match value {
        Ok(entry) => {
            if entry.file_type().is_file() {
                let buf = entry.path().to_path_buf();
                let ext = buf.extension();
                let ext_str = ext.and_then(|v| v.to_str());

                match ext_str {
                    Some("c") | Some("cpp") => Some(buf),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut out_file = PathBuf::from(out_dir);
    out_file.push("bverex");

    let files: Vec<_> = walkdir::WalkDir::new("rex/src/")
        .into_iter()
        .filter_map(match_c_files)
        .collect();

    cc::Build::new()
        .files(files)
        .include("rex/src/")
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std=c++17")
        .compile(out_file.to_str().unwrap());
}
