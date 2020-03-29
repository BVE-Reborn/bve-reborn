use bve::load::mesh::load_mesh_from_file;
use bve_render::{ObjectHandle, Renderer};
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

fn load_and_add(renderer: &mut Renderer) -> Vec<ObjectHandle> {
    let mesh = load_mesh_from_file(
        "C:/Users/connor/AppData/Roaming/openBVE/LegacyContent/Train/R46 2014 (8 Car)/Cars/Body/BodyA.b3d",
    )
    .unwrap();

    assert!(mesh.errors.is_empty(), "{:#?}", mesh);

    mesh.meshes
        .into_iter()
        .map(|mesh| renderer.add_object(Vector3::new(0.0, 0.0, 0.0), &mesh.vertices, &mesh.indices))
        .collect()
}

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

    let mut objects_location = Vector3::new(0.0, 0.0, 0.0);
    let objects = load_and_add(&mut renderer);

    // TODO: Do 0.1 second/1 second/5 seconds/15 second rolling average
    let mut frame_count = 0_u64;
    let mut frame_times = CircularQueue::with_capacity(1024);
    let mut last_frame_instant = Instant::now();
    let mut last_printed_instant = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            if up {
                objects_location.x += 0.01;
            }
            if down {
                objects_location.x -= 0.01;
            }
            if left {
                objects_location.y += 0.01;
            }
            if right {
                objects_location.y -= 0.01;
            }

            for object in &objects {
                renderer.set_location(&object, objects_location).unwrap();
            }

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

            block_on(async {
                renderer.render().await;
            });
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        _ => {}
    })
}
