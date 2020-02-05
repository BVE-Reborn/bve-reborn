pub use root_wrapper::root::*;

pub mod game;

mod root_wrapper {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
