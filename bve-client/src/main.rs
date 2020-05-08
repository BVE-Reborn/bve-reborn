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
use bve::{load::mesh::Vertex, runtime, runtime::Location};
use bve_render::{
    DebugMode, LightDescriptor, MSAASetting, MeshHandle, OITNodeCount, ObjectHandle, PointLight, Renderer,
    RendererStatistics, TextureHandle, Vsync,
};
use cgmath::{ElementWise, InnerSpace, Vector3};
use image::RgbaImage;
use imgui::{im_str, FontSource};
use itertools::Itertools;
use nalgebra_glm::{make_vec3, Vec3};
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
    platform::desktop::EventLoopExtDesktop,
    window::{Window, WindowBuilder},
};

mod platform;

struct Client {
    renderer: Renderer,
}

impl Client {
    async fn new(
        window: &Window,
        imgui_context: &mut imgui::Context,
        oit_node_count: OITNodeCount,
        samples: MSAASetting,
        vsync: Vsync,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            renderer: Renderer::new(window, imgui_context, oit_node_count, samples, vsync).await,
        }))
    }
}

impl runtime::Client for Client {
    type ObjectHandle = ObjectHandle;
    type MeshHandle = MeshHandle;
    type TextureHandle = TextureHandle;

    fn add_object(&mut self, location: Vec3, mesh: &Self::MeshHandle) -> Self::ObjectHandle {
        self.renderer.add_object(location, mesh)
    }

    fn add_object_texture(
        &mut self,
        location: Vec3,
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

    fn remove_object(&mut self, object: &Self::ObjectHandle) {
        self.renderer.remove_object(object)
    }

    fn remove_mesh(&mut self, mesh: &Self::MeshHandle) {
        self.renderer.remove_mesh(mesh)
    }

    fn remove_texture(&mut self, texture: &Self::TextureHandle) {
        self.renderer.remove_texture(texture)
    }

    fn set_camera_location(&mut self, location: Vec3) {
        self.renderer.set_camera_location(location);
    }

    fn set_object_location(&mut self, object: &Self::ObjectHandle, location: Vec3) {
        self.renderer.set_location(object, location);
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
struct Background {
    path: std::path::PathBuf,
    repeats: f32,
}

#[derive(Deserialize)]
struct Loading {
    background: Background,
    objects: Vec<Object>,
}

fn client_main() {
    let mut event_loop = EventLoop::new();

    env_logger::init();

    let window = {
        let mut builder = WindowBuilder::new();
        builder = builder.with_title("BVE-Reborn");
        builder.build(&event_loop).expect("Could not build window")
    };

    let mut window_size = window.inner_size();
    let mut grabber = grabber::Grabber::new(&window, true);

    // Setup imgui
    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, imgui_winit_support::HiDpiMode::Default);
    imgui.set_ini_filename(None);
    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(imgui::FontConfig {
            oversample_h: 3,
            oversample_v: 1,
            pixel_snap_h: true,
            size_pixels: 13.0,
            ..imgui::FontConfig::default()
        }),
    }]);

    let (mut forward, mut left, mut back, mut right, mut up, mut down, mut shift) =
        (false, false, false, false, false, false, false);

    let mut sample_count = MSAASetting::X1;
    let mut oit_node_count = OITNodeCount::Four;
    let mut vsync = Vsync::Enabled;
    let mut debug_mode = DebugMode::None;
    let client = block_on(async { Client::new(&window, &mut imgui, oit_node_count, sample_count, vsync).await });
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
                        runtime::Location::from_absolute_position(Vector3::new(
                            f32::mul_add(object.offset_x, idx as f32, object.x),
                            0.0,
                            f32::mul_add(object.offset_z, idx as f32, object.z),
                        )),
                        PathBuf::from(object.path.clone()),
                    )
                    .await
            }
        }

        let image_contents = async_std::fs::read(&loading.background.path)
            .await
            .expect("Could not load background image");
        let rgba = image::load_from_memory(&image_contents)
            .expect("Could not load background image")
            .into_rgba();
        let mut client = client.lock().await;
        let handle = client.renderer.add_texture(&rgba);
        client.renderer.set_skybox_image(&handle, loading.background.repeats);

        client.renderer.add_light(LightDescriptor::Point(PointLight {
            location: make_vec3(&[0.0, 0.0, 0.0]),
            radius: 200.0,
            strength: 100.0,
        }));
    });

    let mut mouse_pitch = 0.0_f32;
    let mut mouse_yaw = 0.0_f32;

    let mut frame_count = 0_u64;
    let mut frame_times = Vec::with_capacity(3000);
    let mut last_frame_instant = Instant::now();
    let mut last_printed_instant = Instant::now();
    let (
        mut low,
        mut percentile_1th,
        mut percentile_5th,
        mut percentile_50th,
        mut percentile_95th,
        mut percentile_99th,
        mut high,
        mut fps,
    ) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);

    let mut renderer_stats = RendererStatistics::default();
    let mut last_renderer_stats_instant = Instant::now();
    let mut last_cursor = None;

    event_loop.run_return(move |event, _, control_flow| {
        match event {
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

                block_on(async {
                    runtime
                        .set_location(Location::from_absolute_position(camera_location))
                        .await;
                });

                block_on(async {
                    runtime.tick().await;
                });
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
                            Scancodes::ESCAPE => *control_flow = ControlFlow::Exit,
                            Scancodes::LALT => {
                                if state == ElementState::Pressed {
                                    grabber.grab(&window, !grabber.get_grabbed());
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
                mouse_yaw += (delta_x / 1000.0) as f32;
                mouse_pitch += (delta_y / 1000.0) as f32;
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
                    let sorted = frame_times.drain(0..).sorted().collect_vec();

                    low = (*sorted.first().expect("Could not get first value")).as_secs_f32() * 1000.0;
                    percentile_1th = sorted[sorted.len() / 100].as_secs_f32() * 1000.0;
                    percentile_5th = sorted[sorted.len() * 5 / 100].as_secs_f32() * 1000.0;
                    percentile_50th = sorted[sorted.len() * 50 / 100].as_secs_f32() * 1000.0;
                    percentile_95th = sorted[sorted.len() * 95 / 100].as_secs_f32() * 1000.0;
                    percentile_99th = sorted[sorted.len() * 99 / 100].as_secs_f32() * 1000.0;
                    high = (*sorted.last().expect("Could not get last value")).as_secs_f32() * 1000.0;

                    let sum: Duration = sorted.iter().sum();
                    let average = sum / (sorted.len() as u32);
                    fps = 1.0 / average.as_secs_f32();

                    frame_times.clear();

                    last_printed_instant = now;
                }
                frame_count += 1;
                last_frame_instant = now;

                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");

                let frame = imgui.frame();

                {
                    let window = imgui::Window::new(im_str!("Renderer"));
                    window
                        .always_auto_resize(true)
                        .position([25.0, 25.0], imgui::Condition::FirstUseEver)
                        .build(&frame, || {
                            frame.text(format!("Frame Rate: {:.0}fps", fps));
                            frame.text(im_str!("Frame Times (last 1s):"));
                            frame.text(format!("         Average: {:.1}ms", 1000.0 / fps));
                            frame.text(format!("  0th percentile: {:.1}ms", low));
                            frame.text(format!("  1st percentile: {:.1}ms", percentile_1th));
                            frame.text(format!("  5th percentile: {:.1}ms", percentile_5th));
                            frame.text(format!(" 50th percentile: {:.1}ms", percentile_50th));
                            frame.text(format!(" 95th percentile: {:.1}ms", percentile_95th));
                            frame.text(format!(" 99th percentile: {:.1}ms", percentile_99th));
                            frame.text(format!("100th percentile: {:.1}ms", high));
                            frame.separator();
                            frame.text(im_str!("Resources:"));
                            frame.text(format!("Objects: {}", renderer_stats.objects));
                            frame.text(format!("Meshes: {}", renderer_stats.meshes));
                            frame.text(format!("Textures: {}", renderer_stats.textures));
                            frame.separator();
                            frame.text(im_str!("Visible Objects:"));
                            frame.text(format!("Opaque: {}", renderer_stats.visible_opaque_objects));
                            frame.text(format!("Transparent: {}", renderer_stats.visible_transparent_objects));
                            frame.text(format!("Total: {}", renderer_stats.total_visible_objects));
                            frame.separator();
                            frame.text(im_str!("Draws:"));
                            frame.text(format!("Opaque: {}", renderer_stats.opaque_draws));
                            frame.text(format!("Transparent: {}", renderer_stats.transparent_draws));
                            frame.text(format!("Total: {}", renderer_stats.total_draws));
                            frame.separator();
                            frame.text(im_str!("Timings:"));
                            frame.text(format!(
                                "Skybox Update: {:.3}ms",
                                renderer_stats.compute_skybox_update_time
                            ));
                            frame.text(format!(
                                "Object Distances: {:.3}ms",
                                renderer_stats.compute_object_distance_time
                            ));
                            frame.text(format!(
                                "Collect Object Refs: {:.3}ms",
                                renderer_stats.collect_object_refs_time
                            ));
                            frame.text(format!(
                                "Frustum Culling: {:.3}ms",
                                renderer_stats.compute_frustum_culling_time
                            ));
                            frame.text(format!(
                                "Object Sorting: {:.3}ms",
                                renderer_stats.compute_object_sorting_time
                            ));
                            frame.text(format!(
                                "Update Matrices: {:.3}ms",
                                renderer_stats.compute_uniforms_time
                            ));
                            frame.text(format!("Main Render: {:.3}ms", renderer_stats.render_main_cpu_time));
                            frame.text(format!("imgui Render: {:.3}ms", renderer_stats.render_imgui_cpu_time));
                            frame.text(format!("wgpu Render: {:.3}ms", renderer_stats.render_wgpu_cpu_time));
                            frame.separator();
                            frame.text(format!("Total: {:.3}ms", renderer_stats.total_renderer_tick_time));
                            frame.separator();
                            frame.text("Settings");
                            let mut current_vsync = vsync.into_selection_boolean();
                            let res = frame.checkbox(im_str!("Vsync"), &mut current_vsync);
                            if res {
                                vsync = Vsync::from_selection_boolean(current_vsync);
                                block_on(async { client.lock().await.renderer.set_vsync(vsync) });
                            }
                            let mut current_msaa = sample_count.into_selection_integer();
                            if imgui::ComboBox::new(im_str!("Antialiasing"))
                                .flags(imgui::ComboBoxFlags::NO_PREVIEW)
                                .build_simple_string(&frame, &mut current_msaa, &[
                                    im_str!("MSAAx1"),
                                    im_str!("MSAAx2"),
                                    im_str!("MSAAx4"),
                                    im_str!("MSAAx8"),
                                ])
                            {
                                sample_count = MSAASetting::from_selection_integer(current_msaa);
                                block_on(async { client.lock().await.renderer.set_samples(sample_count) });
                            };

                            let mut current_transparency = sample_count.into_selection_integer();
                            if imgui::ComboBox::new(im_str!("Transparency Depth"))
                                .flags(imgui::ComboBoxFlags::NO_PREVIEW)
                                .build_simple_string(&frame, &mut current_transparency, &[
                                    im_str!("4"),
                                    im_str!("8"),
                                    im_str!("16"),
                                    im_str!("32"),
                                ])
                            {
                                oit_node_count = OITNodeCount::from_selection_integer(current_transparency);
                                block_on(async { client.lock().await.renderer.set_oit_node_count(oit_node_count) });
                            };

                            let mut current_debug = debug_mode.into_selection_integer();
                            if imgui::ComboBox::new(im_str!("Debug View"))
                                .flags(imgui::ComboBoxFlags::NO_PREVIEW)
                                .build_simple_string(&frame, &mut current_debug, &[
                                    im_str!("None"),
                                    im_str!("Frustums"),
                                    im_str!("Frustum Addressing Verification"),
                                    im_str!("Per-Pixel Light Count"),
                                ])
                            {
                                debug_mode = DebugMode::from_selection_integer(current_debug);
                                block_on(async { client.lock().await.renderer.set_debug(debug_mode) });
                            };
                        });
                }

                let current_cursor = frame.mouse_cursor();
                if last_cursor != current_cursor && !grabber.get_grabbed() {
                    last_cursor = current_cursor;
                    platform.prepare_render(&frame, &window);
                }

                let tmp_renderer_stats = block_on(async { client.lock().await.renderer.render(Some(frame)).await });
                if now - last_renderer_stats_instant >= Duration::from_millis(200) {
                    last_renderer_stats_instant = now;
                    renderer_stats = tmp_renderer_stats;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        };
        if !grabber.get_grabbed() {
            platform.handle_event(imgui.io_mut(), &window, &event)
        }
    });
}

fn main() {
    let result = catch_unwind(client_main);

    if let Err(..) = result {
        println!("Fatal Error. Copy the above text and report the issue. Press enter to close.");
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).expect("Could not read line");
    }
}
