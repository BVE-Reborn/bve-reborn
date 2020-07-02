use bve_build::*;
use std::process::exit;

const HELP_MESSAGE: &str = r#"Usage: cargo run --bin bve-build -- [args...]
BVE-Reborn build tool.

General:
  -h, --help       Display this help message
  --color=[value]  Pass --color=[value] to all cargo calls
  
Tasks:
  --build     Build bve-reborn
  --cbindgen  Build bve-native C and C++ headers (requires cbindgen)
  --shaderc   Build spirv shaders (requires glslc [from shaderc])

Build Options:
  --debug   Build bve in debug mode
  
Libraries:
  If none of these are specified, builds everything
  --core    Build bve-core
  --client  Build bve-client
  --native  Build bve-native
"#;

fn main() {
    let mut args = pico_args::Arguments::from_env();

    let mut options: Options = Options {
        color: args.opt_value_from_str("--color").unwrap(),
        help: args.contains(["-h", "--help"]),
        debug: args.contains("--debug"),
        build: args.contains("--build"),
        cbindgen: args.contains("--cbindgen"),
        shaderc: args.contains("--shaderc"),
        core: args.contains("--core"),
        client: args.contains("--client"),
        native: args.contains("--native"),
    };

    if let Err(pico_args::Error::UnusedArgsLeft(args)) = args.finish() {
        println!("Unrecognized arguments: {}", args.join(", "));
        options.help = true;
    }

    if options.help {
        println!("{}", HELP_MESSAGE);
        exit(1);
    }

    let all =
        !(options.cbindgen || options.build || options.shaderc || options.native || options.client || options.core);
    let should_build = options.build || options.native || options.client || options.core || all;

    if options.shaderc || options.client || all {
        // build_shaders();
    }

    if should_build {
        build(&options);
    }

    if options.cbindgen || options.native || all {
        generate_c_bindings();
    }
}
