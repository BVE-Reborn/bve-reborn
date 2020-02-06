use crate::root_wrapper::root::*;

pub struct UnsafeGameBuilder {
    pub data: *mut ::core::ffi::c_void,
    pub build:
        unsafe fn(data: *mut ::core::ffi::c_void, interface: &mut rx::render::frontend::interface) -> *mut bve::game,
}

#[bve_derive::c_interface]
pub unsafe extern "C" fn create(
    game_builder: Box<UnsafeGameBuilder>,
    interface: &mut rx::render::frontend::interface,
) -> *mut rx::game {
    (game_builder.build)(game_builder.data, interface) as *mut rx::game
}
