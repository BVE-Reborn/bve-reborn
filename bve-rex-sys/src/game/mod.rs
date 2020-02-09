use crate::root_wrapper::root::*;

pub struct UnsafeGameBuilder {
    pub data: *mut ::core::ffi::c_void,
    pub build:
        unsafe fn(data: *mut ::core::ffi::c_void, interface: &mut rx::render::frontend::interface) -> *mut bve::game,
}

// noinspection RsBorrowChecker
#[bve_derive::c_interface]
pub unsafe extern "C" fn create(
    game_builder: *mut UnsafeGameBuilder,
    interface: &mut rx::render::frontend::interface,
) -> *mut rx::game {
    ((*game_builder).build)((*game_builder).data, interface) as *mut rx::game
}

#[bve_derive::c_interface]
pub unsafe extern "C" fn destroy(game: Box<bve::game>) {
    (game.m_dtor.unwrap())(game.m_self);
}
