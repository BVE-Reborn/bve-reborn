use include_dir::Dir;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, Mutex},
};
use wgpu::{read_spirv, Device, ShaderModule};

const COMPILED_SHADERS: Dir<'static> = include_dir::include_dir!("shaders/spirv");

static SHADER_MODULES: Lazy<Mutex<HashMap<String, Arc<ShaderModule>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn find_shader_module(device: &Device, name: &str) -> Arc<ShaderModule> {
    let mut map = SHADER_MODULES.lock().expect("Could not lock mutex");
    Arc::clone(map.entry(name.to_string()).or_insert_with(|| {
        let source = COMPILED_SHADERS
            .get_file(&name)
            .unwrap_or_else(|| panic!("Shader {} not found", name))
            .contents();
        let spirv = read_spirv(Cursor::new(source)).expect("Could not read shader spirv");
        let shader = device.create_shader_module(&spirv);
        Arc::new(shader)
    }))
}

#[macro_export]
#[doc(hidden)]
#[cfg(debug_assertions)]
macro_rules! spirv_suffix {
    () => {
        ".spv"
    };
}

#[macro_export]
#[doc(hidden)]
#[cfg(not(debug_assertions))]
macro_rules! spirv_suffix {
    () => {
        ".spv.opt"
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! shader {
    ($device:expr; $shader_name:ident - $ty:ident$(: $($name:ident $($eq:tt $value:expr)?);*)?) => {{
        use itertools::Itertools;
        // Please do not read this, it's fun
        $crate::shader::find_shader_module($device, &format!(concat!(stringify!($shader_name), "{}", shader!(@@$ty)), (&[$($(shader!(@$name $($eq $value)?)),*)?] as &[String]).iter().join("")))
    }};
    (@$name:ident = $value:expr) => {
        format!(concat!("_", stringify!($name), "_{}"), $value)
    };
    (@$name:ident) => {
        format!(concat!("_U", stringify!($name)))
    };
    (@@vert) => {
        concat!(".vs", spirv_suffix!())
    };
    (@@vertex) => {
        concat!(".vs", spirv_suffix!())
    };
    (@@frag) => {
        concat!(".fs", spirv_suffix!())
    };
    (@@fragment) => {
        concat!(".fs", spirv_suffix!())
    };
    (@@comp) => {
        concat!(".cs", spirv_suffix!())
    };
    (@@compute) => {
        concat!(".cs", spirv_suffix!())
    };
}
