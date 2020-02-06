use bve_rex_sys::*;

trait GameBuilder: Sized {
    type Built: Game;

    fn build(&mut self, interface: &mut rx::render::frontend::interface) -> Box<Self::Built>;
}

trait Game: Sized {
    fn on_init(&mut self) -> bool;
    fn on_slice(&mut self, input: &mut rx::input::input) -> rx::game_status;
    fn on_resize(&mut self, dimensions: &rx::math::vec2z);
}

unsafe fn build_wrapper<B>(
    data: *mut core::ffi::c_void,
    interface: &mut rx::render::frontend::interface,
) -> *mut bve::game
where
    B: GameBuilder,
{
    let data = data as *mut B;
    let built: Box<B::Built> = (*data).build(interface);
    Box::into_raw(Box::new(bve::game::new(
        Box::into_raw(built) as *mut core::ffi::c_void,
        Some(B::Built::on_init as fn(this: *mut core::ffi::c_void) -> bool),
        Some(B::Built::on_slice as fn(this: *mut core::ffi::c_void, input: &mut rx::input::input) -> rx::game_status),
        Some(B::Built::on_resize as fn(this: *mut core::ffi::c_void, dimensions: &rx::math::vec2z)),
        Some(drop_game::<B::Built>),
    )))
}

unsafe extern "C" fn drop_game<G>(this: *mut core::ffi::c_void)
where
    G: Game,
{
    Box::from_raw(this as *mut G);
}

fn start<B>(builder: Box<B>)
where
    B: GameBuilder,
{
    let builder = game::UnsafeGameBuilder {
        data: Box::into_raw(builder) as *mut core::ffi::c_void,
        build: build_wrapper::<B>,
    };
}
