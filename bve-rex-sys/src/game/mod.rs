use crate::root_wrapper::root::*;

#[bve_derive::c_interface]
pub unsafe extern "C" fn create(_interface: &mut rx::render::frontend::interface) -> *mut rx::game {
    unimplemented!()
}
