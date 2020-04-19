#![feature(clamp, tau_constant)]
// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
// Rustdoc Warnings
#![deny(intra_doc_link_resolution_failure)]
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
// Annoying regular clippy warnings
#![allow(clippy::cast_lossless)] // Annoying
#![allow(clippy::cast_sign_loss)] // Annoying
#![allow(clippy::cast_precision_loss)] // Annoying
#![allow(clippy::cast_possible_truncation)] // Annoying
#![allow(clippy::cognitive_complexity)] // This is dumb
#![allow(clippy::too_many_lines)] // This is also dumb
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::as_conversions)]
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::fallible_impl_from)] // This fails horribly when you try to panic in a macro inside a From impl
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::let_underscore_must_use)]
#![allow(clippy::match_bool)] // prettier
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)] // Cargo deny's job
#![allow(clippy::multiple_inherent_impl)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::option_expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::print_stdout)]
#![allow(clippy::result_expect_used)]
#![allow(clippy::similar_names)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::string_add)]
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]
#![allow(clippy::wildcard_imports)]

use crate::platform::*;
use async_std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    task::block_on,
};
use bve::{load::mesh::Vertex, runtime};
use bve_render::{MSAASetting, MeshHandle, ObjectHandle, Renderer, TextureHandle};
use cgmath::{ElementWise, InnerSpace, Vector3};
use circular_queue::CircularQueue;
use image::RgbaImage;
use itertools::Itertools;
use num_traits::Zero;
use serde::Deserialize;
use std::{
    fs::File,
    io::BufReader,
    panic::catch_unwind,
    time::{Duration, Instant},
};
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod platform;

struct Client {
    renderer: Renderer,
}

impl Client {
    async fn new(window: &Window, samples: MSAASetting) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            renderer: Renderer::new(window, samples).await,
        }))
    }
}

impl runtime::Client for Client {
    type ObjectHandle = ObjectHandle;
    type MeshHandle = MeshHandle;
    type TextureHandle = TextureHandle;

    fn add_object(&mut self, location: Vector3<f32>, mesh: &Self::MeshHandle) -> Self::ObjectHandle {
        self.renderer.add_object(location, mesh)
    }

    fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
        mesh: &Self::MeshHandle,
        texture: &Self::TextureHandle,
    ) -> Self::ObjectHandle {
        self.renderer.add_object_texture(location, mesh, texture)
    }

    fn add_mesh(&mut self, mesh_verts: Vec<Vertex>, indices: &[usize]) -> Self::MeshHandle {
        self.renderer.add_mesh(mesh_verts, indices)
    }

    fn add_texture(&mut self, image: &RgbaImage) -> Self::TextureHandle {
        self.renderer.add_texture(image)
    }
}

#[derive(Deserialize)]
struct Object {
    path: std::path::PathBuf,
    count: usize,
    x: f32,
    z: f32,
    offset_x: f32,
    offset_z: f32,
}

#[derive(Deserialize)]
struct Loading {
    objects: Vec<Object>,
}

fn client_main() {
    let event_loop = EventLoop::new();

    env_logger::init();

    let window = {
        let mut builder = WindowBuilder::new();
        builder = builder.with_title("BVE-Reborn");
        builder.build(&event_loop).expect("Could not build window")
    };

    let mut window_size = window.inner_size();
    let mut grabber = grabber::Grabber::new(&window, true);

    let (mut forward, mut left, mut back, mut right, mut up, mut down, mut shift) =
        (false, false, false, false, false, false, false);

    let mut sample_count = MSAASetting::X1;
    let client = block_on(async { Client::new(&window, sample_count).await });
    let runtime = runtime::Runtime::new(Arc::clone(&client));

    let mut camera_location = Vector3::new(-7.0, 3.0, 0.0);
    block_on(async {
        client
            .lock()
            .await
            .renderer
            .set_camera_orientation(0.0, std::f32::consts::FRAC_PI_2)
    });

    let path = PathBuf::from(std::env::args().nth(1).expect("Must pass filename as first argument"));
    let loading: Loading = serde_json::from_reader(BufReader::new(File::open(path).expect("Could not read file")))
        .expect("Could not parse");

    block_on(async {
        for object in loading.objects {
            for idx in 0..object.count {
                runtime
                    .add_static_object(
                        runtime::Location {
                            chunk: runtime::ChunkAddress::new(0, 0),
                            offset: runtime::ChunkOffset::new(
                                f32::mul_add(object.offset_x, idx as f32, object.x),
                                0.0,
                                f32::mul_add(object.offset_z, idx as f32, object.z),
                            ),
                        },
                        PathBuf::from(object.path.clone()),
                    )
                    .await
            }
        }
    });

    let mut mouse_pitch = 0.0_f32;
    let mut mouse_yaw = 0.0_f32;

    // TODO: Do 0.1 second/1 second/5 seconds/15 second rolling average
    let mut frame_count = 0_u64;
    let mut frame_times = CircularQueue::with_capacity(50);
    let mut last_frame_instant = Instant::now();
    let mut last_printed_instant = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            let last_frame_time = frame_times
                .iter()
                .map(Duration::clone)
                .next()
                .unwrap_or_else(|| Duration::from_secs(0));

            grabber.tick(&window, window_size);

            let speed = if shift { 20.0 } else { 2.0 };

            let raw_dir_vec: Vector3<f32> = Vector3::new(
                if left {
                    -1.0
                } else if right {
                    1.0
                } else {
                    0.0
                },
                if up {
                    1.0
                } else if down {
                    -1.0
                } else {
                    0.0
                },
                if forward {
                    1.0
                } else if back {
                    -1.0
                } else {
                    0.0
                },
            );
            let dir_vec = if raw_dir_vec.is_zero() {
                Vector3::zero()
            } else {
                raw_dir_vec.normalize_to(speed)
            } * last_frame_time.as_secs_f32();

            camera_location = camera_location.add_element_wise(dir_vec);

            block_on(async { client.lock().await.renderer.set_camera_location(camera_location) });

            block_on(async { runtime.tick().await });
            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            window_size = size;
            block_on(async { client.lock().await.renderer.resize(size) });
        }
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { scancode, state, .. },
                    ..
                },
            ..
        } => {
            println!("scancode: 0x{:x}", scancode);
            *match scancode {
                Scancodes::W => &mut forward,
                Scancodes::A => &mut left,
                Scancodes::S => &mut back,
                Scancodes::D => &mut right,
                Scancodes::Q => &mut up,
                Scancodes::Z => &mut down,
                Scancodes::SHIFT => &mut shift,
                _ => {
                    match scancode {
                        // Esc
                        Scancodes::ESCAPE => *control_flow = ControlFlow::Exit,
                        // left alt
                        Scancodes::LALT => {
                            if state == ElementState::Pressed {
                                grabber.grab(&window, !grabber.get_grabbed());
                            }
                        }
                        // comma
                        Scancodes::COMMA => {
                            if state == ElementState::Pressed {
                                sample_count = sample_count.decrement();
                                println!("MSAA: x{}", sample_count as u32);
                                block_on(async { client.lock().await.renderer.set_samples(sample_count) });
                            }
                        }
                        // period
                        Scancodes::PERIOD => {
                            if state == ElementState::Pressed {
                                sample_count = sample_count.increment();
                                println!("MSAA: x{}", sample_count as u32);
                                block_on(async { client.lock().await.renderer.set_samples(sample_count) });
                            }
                        }
                        _ => {}
                    }
                    return;
                }
            } = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            };
        }
        Event::DeviceEvent {
            event:
                DeviceEvent::MouseMotion {
                    delta: (delta_x, delta_y),
                    ..
                },
            ..
        } => {
            use std::f32::consts::TAU;
            if !grabber.get_grabbed() {
                return;
            }
            mouse_yaw += (-delta_x / 1000.0) as f32;
            mouse_pitch += (-delta_y / 1000.0) as f32;
            if mouse_yaw < 0.0 {
                mouse_yaw += TAU;
            } else if mouse_yaw >= TAU {
                mouse_yaw -= TAU;
            }
            mouse_pitch = mouse_pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + 0.0001,
                std::f32::consts::FRAC_PI_2 - 0.0001,
            );

            block_on(async {
                client
                    .lock()
                    .await
                    .renderer
                    .set_camera_orientation(mouse_pitch, mouse_yaw)
            });
        }
        Event::RedrawRequested(_) => {
            let now = Instant::now();
            let duration = now - last_frame_instant;
            frame_times.push(duration);

            if now - last_printed_instant >= Duration::from_secs(1) {
                let sorted = frame_times.iter().map(Duration::clone).sorted().collect_vec();

                let low = *sorted.first().expect("Could not get first value");
                let percentile_1th = sorted[sorted.len() / 100];
                let percentile_5th = sorted[sorted.len() * 5 / 100];
                let percentile_50th = sorted[sorted.len() * 50 / 100];
                let percentile_95th = sorted[sorted.len() * 95 / 100];
                let percentile_99th = sorted[sorted.len() * 99 / 100];
                let high = *sorted.last().expect("Could not get last value");

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

            block_on(async { client.lock().await.renderer.render().await });
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        _ => {}
    })
}

fn main() {
    let result = catch_unwind(client_main);

    if let Err(..) = result {
        println!("Fatal Error. Copy the above text and report the issue. Press enter to close.");
        let mut _s = String::new();
        std::io::stdin().read_line(&mut _s);
    }
}
