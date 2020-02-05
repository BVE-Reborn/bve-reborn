pub use root_wrapper::root::*;

mod root_wrapper {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
