use bve_render::{vertex, Renderer, Vertex};
use cgmath::Vector3;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const VERTEX_DATA: [Vertex; 24] = [
    // top (0, 0, 1)
    vertex([-1, -1, 1], [0, 0]),
    vertex([1, -1, 1], [1, 0]),
    vertex([1, 1, 1], [1, 1]),
    vertex([-1, 1, 1], [0, 1]),
    // bottom (0, 0, -1)
    vertex([-1, 1, -1], [1, 0]),
    vertex([1, 1, -1], [0, 0]),
    vertex([1, -1, -1], [0, 1]),
    vertex([-1, -1, -1], [1, 1]),
    // right (1, 0, 0)
    vertex([1, -1, -1], [0, 0]),
    vertex([1, 1, -1], [1, 0]),
    vertex([1, 1, 1], [1, 1]),
    vertex([1, -1, 1], [0, 1]),
    // left (-1, 0, 0)
    vertex([-1, -1, 1], [1, 0]),
    vertex([-1, 1, 1], [0, 0]),
    vertex([-1, 1, -1], [0, 1]),
    vertex([-1, -1, -1], [1, 1]),
    // front (0, 1, 0)
    vertex([1, 1, -1], [1, 0]),
    vertex([-1, 1, -1], [0, 0]),
    vertex([-1, 1, 1], [0, 1]),
    vertex([1, 1, 1], [1, 1]),
    // back (0, -1, 0)
    vertex([1, -1, 1], [0, 0]),
    vertex([-1, -1, 1], [1, 0]),
    vertex([-1, -1, -1], [1, 1]),
    vertex([1, -1, -1], [0, 1]),
];

const INDEX_DATA: [u16; 36] = [
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 10, 11, 8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
];

async fn async_main() {
    let event_loop = EventLoop::new();

    env_logger::init();

    let window = {
        let mut builder = WindowBuilder::new();
        builder = builder.with_title("BVE-Reborn");
        let window = builder.build(&event_loop).unwrap();
        window
    };

    let mut renderer = Renderer::new(&window).await;

    renderer.add_object(Vector3::new(0.0, 0.0, 3.0), &VERTEX_DATA, &INDEX_DATA);

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => window.request_redraw(),
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => renderer.resize(size),
        Event::RedrawRequested(_) => {
            renderer.render();
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        _ => {}
    })
}

fn main() {
    futures::executor::block_on(async_main())
}
