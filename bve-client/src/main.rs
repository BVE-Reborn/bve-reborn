use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

async fn async_main() {
    let event_loop = EventLoop::new();

    env_logger::init();

    let (window, size) = {
        let mut builder = WindowBuilder::new();
        builder = builder.with_title("BVE-Reborn");
        let window = builder.build(&event_loop).unwrap();
        let size = window.inner_size();
        (window, size)
    };


}

fn main() {
    futures::executor::block_on(async_main())
}
