use include_dir::Dir;
use log::debug;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wgpu::{Device, ShaderModule, ShaderModuleSource};

#[cfg(debug_assertions)]
const COMPILED_SHADERS: Dir<'static> = include_dir::include_dir!("shaders/spirv-debug");
#[cfg(not(debug_assertions))]
const COMPILED_SHADERS: Dir<'static> = include_dir::include_dir!("shaders/spirv");

static SHADER_MODULES: Lazy<Mutex<HashMap<String, Arc<ShaderModule>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn find_shader_module(device: &Device, name: &str) -> Arc<ShaderModule> {
    debug!("Finding shader {}", name);
    let mut map = SHADER_MODULES.lock().expect("Could not lock mutex");
    Arc::clone(map.entry(name.to_string()).or_insert_with(|| {
        debug!("Shader {} not built, building", name);
        let source = COMPILED_SHADERS
            .get_file(&name)
            .unwrap_or_else(|| panic!("Shader {} not found", name))
            .contents()
            .to_vec();
        let shader = device.create_shader_module(ShaderModuleSource::SpirV(bytemuck::cast_slice(&source)));
        Arc::new(shader)
    }))
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
        ".vs.spv"
    };
    (@@vertex) => {
        ".vs.spv"
    };
    (@@frag) => {
        ".fs.spv"
    };
    (@@fragment) => {
       ".fs.spv"
    };
    (@@comp) => {
        ".cs.spv"
    };
    (@@compute) => {
        ".cs.spv"
    };
}
