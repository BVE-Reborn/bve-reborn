use bve_render::{vertex, Renderer, Vertex};
use cgmath::Vector3;
use circular_queue::CircularQueue;
use futures::executor::block_on;
use itertools::Itertools;
use std::time::{Duration, Instant};
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent},
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

fn main() {
    let event_loop = EventLoop::new();

    env_logger::init();

    let window = {
        let mut builder = WindowBuilder::new();
        builder = builder.with_title("BVE-Reborn");
        let window = builder.build(&event_loop).unwrap();
        window
    };

    let (mut up, mut left, mut down, mut right) = (false, false, false, false);

    let mut renderer = block_on(async { Renderer::new(&window).await });

    let mut cube_location = Vector3::new(0.0, 0.0, 0.0);
    let cube = renderer.add_object(cube_location, &VERTEX_DATA, &INDEX_DATA);

    // TODO: Do 0.1 second/1 second/5 seconds/15 second rolling average
    let mut frame_count = 0_u64;
    let mut frame_times = CircularQueue::with_capacity(1024);
    let mut last_frame_instant = Instant::now();
    let mut last_printed_instant = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            if up {
                cube_location.x += 0.01;
            }
            if down {
                cube_location.x -= 0.01;
            }
            if left {
                cube_location.y += 0.01;
            }
            if right {
                cube_location.y -= 0.01;
            }

            block_on(async { renderer.set_location(&cube, cube_location).await }).unwrap();

            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => renderer.resize(size),
        Event::DeviceEvent {
            event: DeviceEvent::Button { button, state },
            ..
        } => {
            println!("{} {:#?}", button, state);
        }
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { scancode, state, .. },
                    ..
                },
            ..
        } => {
            *match scancode {
                // w
                17 => &mut up,
                // a
                30 => &mut left,
                // s
                31 => &mut down,
                // d
                32 => &mut right,
                _ => return,
            } = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            };
        }
        Event::RedrawRequested(_) => {
            let now = Instant::now();
            let duration = now - last_frame_instant;
            frame_times.push(duration);

            if now - last_printed_instant >= Duration::from_secs(1) {
                let sorted = frame_times.iter().map(Duration::clone).sorted().collect_vec();

                let low = *sorted.first().unwrap();
                let percentile_1th = sorted[sorted.len() * 1 / 100];
                let percentile_5th = sorted[sorted.len() * 5 / 100];
                let percentile_50th = sorted[sorted.len() * 50 / 100];
                let percentile_95th = sorted[sorted.len() * 95 / 100];
                let percentile_99th = sorted[sorted.len() * 99 / 100];
                let high = *sorted.last().unwrap();

                let sum: Duration = sorted.iter().sum();
                let average = sum / (sorted.len() as u32);
                let fps = 1.0 / average.as_secs_f32();

                let p = |d: Duration| d.as_secs_f32() * 1000.0;

                println!(
                    "Frame {} ({:.1} fps): ({:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2})",
                    frame_count,
                    fps,
                    p(low),
                    p(percentile_1th),
                    p(percentile_5th),
                    p(percentile_50th),
                    p(percentile_95th),
                    p(percentile_99th),
                    p(high)
                );

                last_printed_instant = now;
            }
            frame_count += 1;
            last_frame_instant = now;

            renderer.render();
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        _ => {}
    })
}
