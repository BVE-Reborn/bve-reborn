use bve_rex_sys::*;
use std::ffi::c_void;
use std::ffi::CString;
use std::mem::transmute;
use std::os::raw::c_char;

pub trait GameBuilder: Sized {
    type Built: Game;

    fn build(&mut self, interface: &mut rx::render::frontend::interface) -> Box<Self::Built>;
}

pub trait Game: Sized {
    fn on_init(&mut self) -> bool;
    fn on_slice(&mut self, input: &mut rx::input::input) -> rx::game_status;
    fn on_resize(&mut self, dimensions: &rx::math::vec2z);
}

// This doesn't need to be extern C nor c_interface(nomangle) as I am calling it from create, which is c_interface
// itself
unsafe fn build_wrapper<B>(data: *mut c_void, interface: &mut rx::render::frontend::interface) -> *mut bve::game
where
    B: GameBuilder,
{
    // This data pointer comes from the start function.
    // This pointer isn't used again, so we are responsible for destroying this.
    let mut data = Box::from_raw(data as *mut B);

    // Once we have the pointer to the builder, call the builder function
    let built: Box<B::Built> = data.build(interface);
    let game = bve::game::new(
        // It will be dropped by the `drop_game` function.
        Box::into_raw(built) as *mut c_void,
        Some(on_init_wrapper::<B::Built>),
        Some(on_slice_wrapper::<B::Built>),
        Some(on_resize_wrapper::<B::Built>),
        Some(drop_game::<B::Built>),
    );
    // Box the game wrapper up and turn it into a pointer.
    // This is dropped in the `destroy` hook provided to rex.
    Box::into_raw(Box::new(game))
}

#[bve_derive::c_interface(mangle)]
unsafe extern "C" fn on_init_wrapper<G>(this: *mut c_void) -> bool
where
    G: Game,
{
    let this = this as *mut G;
    (*this).on_init()
}

#[bve_derive::c_interface(mangle)]
unsafe extern "C" fn on_slice_wrapper<G>(this: *mut c_void, input: *mut rx::input::input) -> rx::game_status
where
    G: Game,
{
    let this = this as *mut G;
    (*this).on_slice(&mut *input)
}

#[bve_derive::c_interface(mangle)]
unsafe extern "C" fn on_resize_wrapper<G>(this: *mut c_void, dimensions: *const rx::math::vec2z)
where
    G: Game,
{
    let this = this as *mut G;
    (*this).on_resize(&*dimensions)
}

#[bve_derive::c_interface(mangle)]
unsafe extern "C" fn drop_game<G>(this: *mut c_void)
where
    G: Game,
{
    // The game is going away, so we must destroy it
    Box::from_raw(this as *mut G);
}

pub fn start<B>(builder: Box<B>)
where
    B: GameBuilder,
{
    let mut builder = game::UnsafeGameBuilder {
        // This box is freed by the build_wrapper function it is passed to.
        data: Box::into_raw(builder) as *mut c_void,
        build: build_wrapper::<B>,
    };

    let mut args: Vec<*mut c_char> = std::env::args()
        .map(|v| CString::new(v).expect("Argv should be c compatable").into_raw())
        .collect();

    unsafe {
        rx_main(
            &mut builder as *mut game::UnsafeGameBuilder as *mut c_void,
            transmute::<
                unsafe extern "C" fn(
                    *mut game::UnsafeGameBuilder,
                    &mut rx::render::frontend::interface,
                ) -> *mut rx::game,
                rx::creator,
            >(game::create),
            transmute::<unsafe extern "C" fn(Box<bve::game>), rx::destructor>(game::destroy),
            args.len() as i32,
            args.as_mut_ptr(),
        );
    }

    args.into_iter().for_each(|ptr| unsafe {
        CString::from_raw(ptr);
    })
}
