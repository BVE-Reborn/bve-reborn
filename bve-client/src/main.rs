#![feature(clamp, tau_constant)]

use bve::load::mesh::load_mesh_from_file;
use bve_render::{MSAASetting, ObjectHandle, Renderer};
use cgmath::{ElementWise, InnerSpace, Vector3, Vector4};
use circular_queue::CircularQueue;
use futures::executor::block_on;
use image::{Rgba, RgbaImage};
use itertools::Itertools;
use num_traits::Zero;
use std::{
    path::Path,
    time::{Duration, Instant},
};
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn load_texture(name: impl AsRef<Path>) -> RgbaImage {
    let base = Path::new("C:/Users/connor/AppData/Roaming/openBVE/LegacyContent/Train/R46 2014 (8 Car)/Cars/Body/");
    let result = base.join(name.as_ref());
    let img = image::open(result).unwrap();
    let mut rgba = img.into_rgba();
    process_texture(&mut rgba);
    rgba
}

fn process_texture(texture: &mut RgbaImage) {
    texture.pixels_mut().for_each(|pix| {
        // Convert pure blue to transparent
        if let Rgba([0x00, 0x00, 0xFF, w]) = pix {
            *w = 0x00
        }
    });
    let width = texture.width() as i32;
    let height = texture.height() as i32;
    let load = |image: &RgbaImage, x: i32, y: i32| {
        if x >= 0 && x < width && y >= 0 && y < height {
            let pix = *image.get_pixel(x as u32, y as u32);
            match pix {
                Rgba([_, _, _, 0x00]) => Vector4::new(0x00 as f32, 0x00 as f32, 0x00 as f32, 0x00 as f32),
                Rgba([x, y, z, _]) => Vector4::new(x as f32, y as f32, z as f32, 0xFF as f32),
            }
        } else {
            Vector4::new(0x00 as f32, 0x00 as f32, 0x00 as f32, 0x00 as f32)
        }
    };
    for x in 0..width {
        for y in 0..height {
            let pix11 = load(&texture, x, y);
            if pix11.w == 0.0 {
                let pix00 = load(&texture, x - 1, y - 1);
                let pix10 = load(&texture, x, y - 1);
                let pix20 = load(&texture, x + 1, y - 1);
                let pix01 = load(&texture, x - 1, y);
                let pix21 = load(&texture, x + 1, y);
                let pix02 = load(&texture, x - 1, y + 1);
                let pix12 = load(&texture, x, y + 1);
                let pix22 = load(&texture, x + 1, y + 1);

                let sum: Vector4<f32> = pix00 + pix01 + pix02 + pix10 + pix12 + pix20 + pix21 + pix22;
                let scale = sum.w / 255.0;
                let avg = Vector3::new(sum.x, sum.y, sum.z) / scale;
                texture.put_pixel(x as u32, y as u32, Rgba([avg.x as u8, avg.y as u8, avg.z as u8, 0x00]))
            }
        }
    }
}

fn load_and_add(renderer: &mut Renderer) -> Vec<ObjectHandle> {
    let mesh = load_mesh_from_file(
        "C:/Users/connor/AppData/Roaming/openBVE/LegacyContent/Train/R46 2014 (8 Car)/Cars/Body/BodyA.b3d",
    )
    .unwrap();

    assert!(mesh.errors.is_empty(), "{:#?}", mesh);

    let texture_handles = mesh
        .textures
        .into_iter()
        .map(|s| {
            let image = load_texture(s);
            renderer.add_texture(&image)
        })
        .collect_vec();

    mesh.meshes
        .into_iter()
        .map(|mesh| {
            let default_handle = Renderer::get_default_texture();
            let handle = if let Some(id) = mesh.texture.texture_id {
                &texture_handles[id]
            } else {
                &default_handle
            };
            let obj = renderer.add_object_texture(
                Vector3::new(0.0, 0.0, 0.0),
                mesh.vertices.clone(),
                &mesh.indices,
                &handle,
            );
            // for i in 1..10 {
            //     let obj = renderer.add_object_texture(
            //         Vector3::new(i as f32 * 3.0, 0.0, 0.0),
            //         mesh.vertices.clone(),
            //         &mesh.indices,
            //         &handle,
            //     );
            // }
            obj
        })
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

    let mut mouse_grabbed = true;
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    let (mut forward, mut left, mut back, mut right, mut up, mut down, mut shift) =
        (false, false, false, false, false, false, false);

    let mut sample_count = MSAASetting::X1;
    let mut renderer = block_on(async { Renderer::new(&window, sample_count).await });

    let mut camera_location = Vector3::new(-200.0, 3.0, 0.0);

    let objects = load_and_add(&mut renderer);

    let mut mouse_pitch = 0.0_f32;
    let mut mouse_yaw = 0.0_f32;

    // TODO: Do 0.1 second/1 second/5 seconds/15 second rolling average
    let mut frame_count = 0_u64;
    let mut frame_times = CircularQueue::with_capacity(1024);
    let mut last_frame_instant = Instant::now();
    let mut last_printed_instant = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            let last_frame_time = frame_times
                .iter()
                .map(Duration::clone)
                .next()
                .unwrap_or_else(|| Duration::from_secs(0));

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

            renderer.set_camera_location(camera_location);

            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => renderer.resize(size, sample_count),
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { scancode, state, .. },
                    ..
                },
            ..
        } => {
            println!("scancode: {}", scancode);
            *match scancode {
                // w
                17 => &mut forward,
                // a
                30 => &mut left,
                // s
                31 => &mut back,
                // d
                32 => &mut right,
                // q
                16 => &mut up,
                // z
                44 => &mut down,
                // shift
                42 => &mut shift,
                _ => {
                    match scancode {
                        // Esc
                        1 => *control_flow = ControlFlow::Exit,
                        // left alt
                        56 => {
                            if state == ElementState::Pressed {
                                if mouse_grabbed {
                                    window.set_cursor_grab(false);
                                    window.set_cursor_visible(true);
                                } else {
                                    window.set_cursor_grab(true);
                                    window.set_cursor_visible(false);
                                }
                                mouse_grabbed = !mouse_grabbed;
                            }
                        }
                        // comma
                        51 => {
                            if state == ElementState::Pressed {
                                sample_count = sample_count.decrement();
                                println!("MSAA: x{}", sample_count as u32);
                                renderer.set_samples(sample_count)
                            }
                        }
                        // period
                        52 => {
                            if state == ElementState::Pressed {
                                sample_count = sample_count.increment();
                                println!("MSAA: x{}", sample_count as u32);
                                renderer.set_samples(sample_count)
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
            if mouse_grabbed == false {
                return;
            }
            use std::f32::consts::TAU;
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

            renderer.set_camera(mouse_pitch, mouse_yaw);
        }
        Event::RedrawRequested(_) => {
            let now = Instant::now();
            let duration = now - last_frame_instant;
            frame_times.push(duration);

            if now - last_printed_instant >= Duration::from_secs(1) {
                let sorted = frame_times.iter().map(Duration::clone).sorted().collect_vec();

                let low = *sorted.first().unwrap();
                let percentile_1th = sorted[sorted.len() / 100];
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
