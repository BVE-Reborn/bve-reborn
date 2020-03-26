use bindgen::{EnumVariation, RustTarget};
use std::path::PathBuf;
use walkdir::{DirEntry, Error};

trait BuildExt {
    /// Enable clang-cl only on windows
    fn enable_clang_cl(&mut self) -> &mut Self;
}

impl BuildExt for cmake::Config {
    fn enable_clang_cl(&mut self) -> &mut Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                let llvm_dir = std::env::var("LLVM_DIR").ok();
                let llvm_dir = llvm_dir.unwrap_or_else(|| String::from("C:\\Program Files\\LLVM\\"));
                let mut clang_path = PathBuf::from(llvm_dir);
                clang_path.push("bin");
                clang_path.push("clang-cl.exe");
                if !clang_path.exists() || !clang_path.is_file() {
                    match which::which("clang-cl") {
                        Ok(path) => clang_path = path,
                        Err(..) => panic!("Rex requires clang-cl on Windows. Please add it to path, set LLVM_DIR to
        the root of your LLVM install or install LLVM to C:/Program Files/LLVM/"),                    }
                }
                // Cause cmake is fuckin stupid
                let fixed_path = clang_path.display().to_string().replace("\\", "/");
                self.define("CMAKE_C_COMPILER", &fixed_path).define("CMAKE_CXX_COMPILER", fixed_path).generator("Ninja")
            } else {
                self
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum FileTypes {
    C,
    Cpp,
    All,
}

impl FileTypes {
    pub fn c(&self) -> bool {
        (*self == Self::C) | (*self == Self::All)
    }

    pub fn cpp(&self) -> bool {
        (*self == Self::Cpp) | (*self == Self::All)
    }

    pub fn headers(&self) -> bool {
        *self == Self::All
    }
}

fn match_c_files(files: FileTypes) -> impl Fn(Result<DirEntry, Error>) -> Option<PathBuf> {
    move |value| match value {
        Ok(entry) => {
            if entry.file_type().is_file() {
                let buf = entry.path().to_path_buf();
                let ext = buf.extension();
                let ext_str = ext.and_then(|v| v.to_str());

                match ext_str {
                    Some("c") if files.c() => Some(buf),
                    Some("cpp") if files.cpp() => Some(buf),
                    Some("h") | Some("hpp") if files.headers() => Some(buf),
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
    cfg_if::cfg_if! {
        if #[cfg(target_env = "mingw")] {
            panic!("bve-rex-sys does not support mingw");
        } else if #[cfg(target_os = "macos")] {
            panic!("bve-rex-sys does not support macos");
        }
    }

    // Announce our include directories
    let mut include_path: PathBuf = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    include_path.push("rex");
    include_path.push("src");
    println!("cargo:include={}", include_path.display());

    walkdir::WalkDir::new("rex/src/")
        .into_iter()
        .filter_map(match_c_files(FileTypes::All))
        .for_each(|p: PathBuf| println!("cargo:rerun-if-changed={}", p.display()));
    walkdir::WalkDir::new("wrapper")
        .into_iter()
        .filter_map(match_c_files(FileTypes::All))
        .for_each(|p: PathBuf| println!("cargo:rerun-if-changed={}", p.display()));
    println!("cargo:rerun-if-changed=CMakeLists.txt");

    run_bindgen();

    let location = cmake::Config::new(".")
        .enable_clang_cl()
        .define("SDL2_INCLUDE_DIR", std::env::var("DEP_SDL2_INCLUDE").unwrap())
        .define("SDL2_LINK_SEARCH", std::env::var("DEP_SDL2_LINK_SEARCH").unwrap())
        .build();

    println!("cargo:rustc-link-search=native={}", location.display());
    println!("cargo:rustc-link-lib=dylib=bverex");
}

fn run_bindgen() {
    let bindings = bindgen::builder()
        .clang_arg("-Irex/src")
        .clang_arg("-std=c++17")
        .header("wrapper/wrapper.hpp")
        .whitelist_type("bve::.*")
        .whitelist_type("rx::game$")
        .whitelist_type("rx::render::frontend::interface$")
        .whitelist_function("create$")
        .whitelist_function("rx_main")
        .opaque_type("rx::concurrency.*")
        .opaque_type("rx::traits.*")
        .no_copy("rx::concurrency.*")
        .rust_target(RustTarget::Nightly)
        .use_core()
        .enable_cxx_namespaces()
        .default_enum_style(EnumVariation::NewType { is_bitfield: true })
        .size_t_is_usize(true)
        .rustfmt_bindings(true)
        .generate()
        .expect("Couldn't generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
