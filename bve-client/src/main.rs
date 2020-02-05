fn main() {
    unsafe {
        sdl2_sys::SDL_Init(sdl2_sys::SDL_INIT_VIDEO);
        bve_rex_sys::rx_main(0, std::ptr::null_mut());
    }
}
