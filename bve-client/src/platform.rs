#[allow(non_snake_case, unused)]
#[cfg(not(target_os = "macos"))]
pub mod Scancodes {
    pub const W: u32 = 0x11;
    pub const A: u32 = 0x1E;
    pub const S: u32 = 0x1F;
    pub const D: u32 = 0x20;
    pub const Q: u32 = 0x10;
    pub const Z: u32 = 0x2C;
    pub const SEMICOLON: u32 = 0x27;
    pub const QUOTE: u32 = 0x28;
    pub const COMMA: u32 = 0x33;
    pub const PERIOD: u32 = 0x34;
    pub const SHIFT: u32 = 0x2A;
    pub const ESCAPE: u32 = 0x01;
    pub const LALT: u32 = 0x38;
}

// https://stackoverflow.com/a/16125341 reference
#[allow(non_snake_case, unused)]
#[cfg(target_os = "macos")]
pub mod Scancodes {
    pub const W: u32 = 0x0D;
    pub const A: u32 = 0x00;
    pub const S: u32 = 0x01;
    pub const D: u32 = 0x02;
    pub const Q: u32 = 0x0C;
    pub const Z: u32 = 0x06;
    pub const SEMICOLON: u32 = 0x29;
    pub const QUOTE: u32 = 0x27;
    pub const COMMA: u32 = 0x2B;
    pub const PERIOD: u32 = 0x2F;
    pub const SHIFT: u32 = 0x38;
    pub const ESCAPE: u32 = 0x35;
    pub const LALT: u32 = 0x3A; // Actually Left Option
}

#[cfg(not(target_os = "linux"))]
pub mod grabber {
    use winit::{dpi::PhysicalSize, window::Window};

    pub struct Grabber {
        grabbed: bool,
    }

    impl Grabber {
        pub fn new(window: &Window, grab: bool) -> Self {
            let mut g = Self { grabbed: grab };
            g.grab(window, grab);
            g
        }

        pub const fn get_grabbed(&self) -> bool {
            self.grabbed
        }

        pub fn grab(&mut self, window: &Window, grab: bool) {
            self.grabbed = grab;
            window.set_cursor_grab(grab).expect("Unable to grab mouse");
            window.set_cursor_visible(!grab);
        }

        #[allow(clippy::missing_const_for_fn, clippy::unused_self)]
        pub fn tick(&self, _window: &Window, _size: PhysicalSize<u32>) {
            // noop on !linux
        }
    }
}

#[cfg(target_os = "linux")]
pub mod grabber {
    use winit::{
        dpi::{PhysicalPosition, PhysicalSize},
        window::Window,
    };

    pub struct Grabber {
        grabbed: bool,
    }

    impl Grabber {
        pub fn new(window: &Window, grab: bool) -> Self {
            window.set_cursor_visible(!grab);
            Self { grabbed: grab }
        }

        pub const fn get_grabbed(&self) -> bool {
            self.grabbed
        }

        pub fn grab(&mut self, window: &Window, grab: bool) {
            self.grabbed = grab;
            window.set_cursor_visible(!grab);
        }

        pub fn tick(&self, window: &Window, size: PhysicalSize<u32>) {
            if self.grabbed {
                window
                    .set_cursor_position(PhysicalPosition {
                        x: size.width / 2,
                        y: size.height / 2,
                    })
                    .expect("Could not set cursor position");
            }
        }
    }
}
