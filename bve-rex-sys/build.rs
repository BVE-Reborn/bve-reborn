use std::path::PathBuf;
use walkdir::{DirEntry, Error};

trait BuildExt {
    /// Enable clang-cl only on windows
    fn enable_clang_cl(&mut self) -> &mut Self;
    /// Platform specific defines
    fn add_defines(&mut self) -> &mut Self;
    /// Enable c++17
    fn enable_cpp17(&mut self) -> &mut Self;
}

impl BuildExt for cc::Build {
    fn enable_clang_cl(&mut self) -> &mut Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                let llvm_dir = std::env::var("LLVM_DIR").ok();
                let llvm_dir = llvm_dir.unwrap_or_else(|| String::from("C:/Program Files/LLVM/"));
                let mut clang_path = PathBuf::from(llvm_dir);
                clang_path.push("bin");
                clang_path.push("clang-cl.exe");
                if !clang_path.exists() || !clang_path.is_file() {
                    panic!("Rex requires clang-cl on Windows. Please set LLVM_DIR to the root of your LLVM install or install LLVM to C:/Program Files/LLVM/")
                }
                self.compiler(clang_path)
            } else {
                self
            }
        }
    }
    fn add_defines(&mut self) -> &mut Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                self.define("_CRT_SECURE_NO_WARNINGS", None)
            } else {
                self
            }
        }
    }
    fn enable_cpp17(&mut self) -> &mut Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                self.flag_if_supported("/std:c++17")
            } else {
                self.flag_if_supported("-std=c++1z")
            }
        }
    }
}

fn match_c_files(enable_headers: bool) -> impl Fn(Result<DirEntry, Error>) -> Option<PathBuf> {
    move |value| match value {
        Ok(entry) => {
            if entry.file_type().is_file() {
                let buf = entry.path().to_path_buf();
                let ext = buf.extension();
                let ext_str = ext.and_then(|v| v.to_str());

                match ext_str {
                    Some("c") | Some("cpp") => Some(buf),
                    Some("h") | Some("hpp") if enable_headers => Some(buf),
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
    #[cfg(target_env = "mingw")]
    {
        panic!("Rex does not support mingw");
    }
    #[cfg(target_os = "macos")]
    {
        panic!("Rex does not support macos");
    }

    let c_sources: Vec<_> = walkdir::WalkDir::new("rex/src/")
        .into_iter()
        .filter_map(match_c_files(false))
        .collect();

    walkdir::WalkDir::new("rex/src/")
        .into_iter()
        .filter_map(match_c_files(true))
        .for_each(|p: PathBuf| println!("cargo:rerun-if-changed={}", p.display()));

    cc::Build::new()
        .enable_clang_cl()
        .include(std::env::var("DEP_SDL2_INCLUDE").unwrap())
        .include("rex/src/")
        .cpp(true)
        .enable_cpp17()
        .add_defines()
        .warnings(false)
        .files(c_sources)
        .compile("bverex");
}
