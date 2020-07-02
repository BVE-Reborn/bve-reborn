use std::path::Path;

fn main() {
    let env = std::env::var("OUT_DIR").unwrap();
    let path = Path::new(&env);
    let shaders = path.join("shaders-include.rs");
    let (shader_dir, release) = if std::env::var("PROFILE").unwrap() == "release" {
        let shader_dir = path.join("shaders/spirv");
        (shader_dir, true)
    } else {
        let shader_dir = path.join("shaders/spirv-debug");
        (shader_dir, false)
    };
    std::fs::write(
        shaders,
        format!(
            r##"const COMPILED_SHADERS: include_dir::Dir<'static> = include_dir::include_dir!(r#"{}"#);"##,
            shader_dir.display()
        ),
    )
    .unwrap();
    bve_build::build_shaders(path, release);
}
