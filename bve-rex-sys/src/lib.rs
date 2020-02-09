pub use root_wrapper::root::*;

pub mod game;

mod root_wrapper {
    #![allow(clippy::missing_safety_doc)]
    #![allow(clippy::should_implement_trait)]
    #![allow(clippy::transmute_ptr_to_ptr)]
    #![allow(clippy::too_many_arguments)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
