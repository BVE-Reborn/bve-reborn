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

pub fn find_shader_module(device: &Device, name: String) -> Arc<ShaderModule> {
    let mut map = SHADER_MODULES.lock().expect("Could not lock mutex");
    Arc::clone(map.entry(name.clone()).or_insert_with(|| {
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
macro_rules! shader {
    ($device:expr; $shader_name:ident - $ty:ident$(: $($name:ident $($eq:tt $value:expr)?);*)?) => {{
        use itertools::Itertools;
        $crate::shader::find_shader_module($device, format!(concat!(stringify!($shader_name), "{}", shader!(@@$ty)), (&[$($(shader!(@$name $($eq $value)?)),*)?] as &[String]).iter().join("")))
    }};
    (@$name:ident = $value:expr) => {
        format!(concat!("_", stringify!($name), "_{}"), $value)
    };
    (@NOT $name:ident) => {
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
