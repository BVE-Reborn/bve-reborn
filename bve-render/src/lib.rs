// +x right
// +y up
// +z away from camera

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
#![allow(clippy::result_expect_used)]
#![allow(clippy::similar_names)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::string_add)]
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]
#![allow(clippy::wildcard_imports)]

use crate::render::Uniforms;
pub use crate::{mesh::MeshHandle, object::ObjectHandle, render::MSAASetting, texture::TextureHandle};
use bve::load::mesh::Vertex as MeshVertex;
use cgmath::{Matrix4, Vector2, Vector3};
use image::RgbaImage;
use indexmap::map::IndexMap;
use itertools::Itertools;
use num_traits::{ToPrimitive, Zero};
use std::{io, mem::size_of};
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};
use zerocopy::{AsBytes, FromBytes};

#[cfg(debug_assertions)]
macro_rules! shader_path {
    ($name:literal, $suffix:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, $suffix, ".spv"))
    };
}

#[cfg(not(debug_assertions))]
macro_rules! shader_path {
    ($name:literal, $suffix:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/shaders/",
            $name,
            $suffix,
            ".spv.opt"
        ))
    };
}

#[cfg(feature = "renderdoc")]
macro_rules! renderdoc {
    ($($tokens:tt)*) => {
        $($tokens)*
    };
}

#[cfg(not(feature = "renderdoc"))]
macro_rules! renderdoc {
    ($($tokens:tt)*) => {};
}

macro_rules! include_shader {
    (vert $name:literal) => {
        shader_path!($name, ".vs")
    };
    (geo $name:literal) => {
        shader_path!($name, ".gs")
    };
    (frag $name:literal) => {
        shader_path!($name, ".fs")
    };
    (comp $name:literal) => {
        shader_path!($name, ".cs")
    };
}

mod camera;
mod compute;
mod mesh;
mod object;
mod render;
mod texture;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, 0.0, //
    0.0, 0.0, -0.5, 0.5, //
    0.0, 0.0, 0.5, 1.0,
);

pub struct Renderer {
    objects: IndexMap<u64, object::Object>,
    object_handle_count: u64,

    mesh: IndexMap<u64, mesh::Mesh>,
    mesh_handle_count: u64,

    textures: IndexMap<u64, texture::Texture>,
    texture_handle_count: u64,

    camera: camera::Camera,
    screen_size: PhysicalSize<u32>,
    samples: render::MSAASetting,

    surface: Surface,
    device: Device,
    queue: Queue,
    swapchain: SwapChain,
    framebuffer: TextureView,
    depth_buffer: TextureView,
    opaque_pipeline: RenderPipeline,
    alpha_pipeline: RenderPipeline,
    pipeline_layout: PipelineLayout,
    texture_bind_group_layout: BindGroupLayout,
    sampler: Sampler,

    vert_shader: ShaderModule,
    frag_shader: ShaderModule,

    transparency_processor: compute::CutoutTransparencyCompute,
    mip_creator: compute::MipmapCompute,

    command_buffers: Vec<CommandBuffer>,
    _renderdoc_capture: bool,
}

impl Renderer {
    pub async fn new(window: &Window, samples: render::MSAASetting) -> Self {
        let screen_size = window.inner_size();

        let surface = Surface::create(window);

        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            BackendBit::VULKAN | BackendBit::METAL,
        )
        .await
        .expect("Could not create Adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                extensions: Extensions {
                    anisotropic_filtering: true,
                },
                limits: Limits::default(),
            })
            .await;

        let swapchain = render::create_swapchain(&device, &surface, screen_size);

        let vs = include_shader!(vert "opaque");
        let vs_module =
            device.create_shader_module(&read_spirv(io::Cursor::new(&vs[..])).expect("Could not read shader spirv"));

        let fs = include_shader!(frag "opaque");
        let fs_module =
            device.create_shader_module(&read_spirv(io::Cursor::new(&fs[..])).expect("Could not read shader spirv"));

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: CompareFunction::Never,
        });

        let framebuffer = render::create_framebuffer(&device, screen_size, samples);
        let depth_buffer = render::create_depth_buffer(&device, screen_size, samples);

        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        component_type: TextureComponentType::Uint,
                        dimension: TextureViewDimension::D2,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                },
            ],
            label: Some("texture and sampler"),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&texture_bind_group_layout],
        });

        let opaque_pipeline = render::create_pipeline(
            &device,
            &pipeline_layout,
            &vs_module,
            &fs_module,
            render::PipelineType::Normal,
            samples,
        );
        let alpha_pipeline = render::create_pipeline(
            &device,
            &pipeline_layout,
            &vs_module,
            &fs_module,
            render::PipelineType::Alpha,
            samples,
        );

        let transparency_processor = compute::CutoutTransparencyCompute::new(&device);
        let mip_creator = compute::MipmapCompute::new(&device);

        // Create the Renderer object early so we can can call methods on it.
        let mut renderer = Self {
            objects: IndexMap::new(),
            object_handle_count: 0,

            mesh: IndexMap::new(),
            mesh_handle_count: 0,

            textures: IndexMap::new(),
            texture_handle_count: 0,

            camera: camera::Camera {
                location: Vector3::new(-6.0, 0.0, 0.0),
                pitch: 0.0,
                yaw: 0.0,
            },
            screen_size,
            samples,

            surface,
            device,
            queue,
            swapchain,
            framebuffer,
            depth_buffer,
            opaque_pipeline,
            alpha_pipeline,
            pipeline_layout,
            texture_bind_group_layout,
            sampler,

            vert_shader: vs_module,
            frag_shader: fs_module,

            transparency_processor,
            mip_creator,

            command_buffers: Vec::new(),
            _renderdoc_capture: false,
        };

        // Default texture is texture handle zero, immediately discard the handle, never to be seen again
        renderer.add_texture(&RgbaImage::from_raw(1, 1, vec![0xff, 0xff, 0xff, 0xff]).expect("Invalid Image"));

        renderer
    }

    pub fn set_location(&mut self, object::ObjectHandle(handle): &object::ObjectHandle, location: Vector3<f32>) {
        let object: &mut object::Object = &mut self.objects[handle];

        object.location = location;
    }

    pub fn resize(&mut self, screen_size: PhysicalSize<u32>) {
        self.framebuffer = render::create_framebuffer(&self.device, screen_size, self.samples);
        self.depth_buffer = render::create_depth_buffer(&self.device, screen_size, self.samples);
        self.screen_size = screen_size;

        self.swapchain = render::create_swapchain(&self.device, &self.surface, screen_size);
    }

    #[must_use]
    pub const fn get_samples(&self) -> render::MSAASetting {
        self.samples
    }

    pub fn set_samples(&mut self, samples: render::MSAASetting) {
        self.framebuffer = render::create_framebuffer(&self.device, self.screen_size, samples);
        self.depth_buffer = render::create_depth_buffer(&self.device, self.screen_size, samples);
        self.opaque_pipeline = render::create_pipeline(
            &self.device,
            &self.pipeline_layout,
            &self.vert_shader,
            &self.frag_shader,
            render::PipelineType::Normal,
            samples,
        );
        self.alpha_pipeline = render::create_pipeline(
            &self.device,
            &self.pipeline_layout,
            &self.vert_shader,
            &self.frag_shader,
            render::PipelineType::Alpha,
            samples,
        );
        self.samples = samples;
    }

    pub async fn render(&mut self) {
        renderdoc! {
            let mut rd = renderdoc::RenderDoc::<renderdoc::V140>::new().expect("Could not initialize renderdoc");
            if self._renderdoc_capture {
                rd.start_frame_capture(std::ptr::null(), std::ptr::null());
            }
        }
        self.compute_object_distances();
        let object_references = Self::sort_objects(&self.objects);
        let (uniform_command_buffer_opt, matrix_buffer_opt) = self.recompute_uniforms(&object_references).await;
        if let Some(uniform_command_buffer) = uniform_command_buffer_opt {
            self.command_buffers.push(uniform_command_buffer);
        }

        let frame = self
            .swapchain
            .get_next_texture()
            .expect("Could not get next swapchain texture");

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: Some("primary") });

        {
            let (attachment, resolve_target) = if self.samples == render::MSAASetting::X1 {
                (&frame.view, None)
            } else {
                (&self.framebuffer, Some(&frame.view))
            };
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment,
                    resolve_target,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.3_f64.powf(1.0 / 2.2),
                        g: 0.3_f64.powf(1.0 / 2.2),
                        b: 0.3_f64.powf(1.0 / 2.2),
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_buffer,
                    depth_load_op: LoadOp::Clear,
                    depth_store_op: StoreOp::Store,
                    stencil_load_op: LoadOp::Clear,
                    stencil_store_op: StoreOp::Store,
                    clear_depth: 0.0,
                    clear_stencil: 0,
                }),
            });

            // If se don't have a matrix buffer we have nothing to render
            if let Some(matrix_buffer) = matrix_buffer_opt.as_ref() {
                let mut current_matrix_offset = 0 as BufferAddress;

                let mut opaque_ended = false;
                rpass.set_pipeline(&self.opaque_pipeline);
                for ((mesh_idx, texture_idx, transparent), group) in &object_references
                    .into_iter()
                    .group_by(|o| (o.mesh, o.texture, o.transparent))
                {
                    if transparent && !opaque_ended {
                        rpass.set_pipeline(&self.alpha_pipeline);
                        opaque_ended = true;
                    }
                    let mesh = &self.mesh[&mesh_idx];
                    let texture_bind = &self.textures[&texture_idx].bind_group;
                    let count = group.count();
                    let matrix_buffer_size = (count * size_of::<Uniforms>()) as BufferAddress;

                    rpass.set_bind_group(0, texture_bind, &[]);
                    rpass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                    rpass.set_vertex_buffer(1, matrix_buffer, current_matrix_offset, matrix_buffer_size);
                    rpass.set_index_buffer(&mesh.index_buffer, 0, 0);
                    rpass.draw_indexed(0..(mesh.index_count as u32), 0, 0..(count as u32));
                    current_matrix_offset += matrix_buffer_size;
                    if current_matrix_offset & 255 != 0 {
                        current_matrix_offset += 256 - (current_matrix_offset & 255)
                    }
                }
            }
        }

        self.command_buffers.push(encoder.finish());

        self.queue.submit(&self.command_buffers);
        self.command_buffers.clear();
        renderdoc! {
            if self._renderdoc_capture {
                rd.end_frame_capture(std::ptr::null(), std::ptr::null());
                self._renderdoc_capture = false;
            }
        }
    }
}
