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
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]
#![allow(clippy::wildcard_imports)]

pub use crate::{
    mesh::MeshHandle, object::ObjectHandle, oit::OITNodeCount, render::MSAASetting, texture::TextureHandle,
};
use crate::{object::perspective_matrix, render::Uniforms};
use bve::load::mesh::Vertex as MeshVertex;
use cgmath::{Deg, Matrix4, Vector2, Vector3};
use image::RgbaImage;
use indexmap::map::IndexMap;
use itertools::Itertools;
use num_traits::{ToPrimitive, Zero};
use std::{mem::size_of, sync::Arc};
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};
use zerocopy::{AsBytes, FromBytes};

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

mod camera;
mod compute;
mod mesh;
mod object;
mod oit;
mod render;
mod screenspace;
mod shader;
mod skybox;
mod texture;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, 0.0, //
    0.0, 0.0, -0.5, 0.0, //
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
    resolution: PhysicalSize<u32>,
    oit_node_count: oit::OITNodeCount,
    samples: render::MSAASetting,

    projection_matrix: Matrix4<f32>,

    surface: Surface,
    device: Device,
    queue: Queue,
    swapchain: SwapChain,
    framebuffer: TextureView,
    depth_buffer: TextureView,
    opaque_pipeline: RenderPipeline,
    pipeline_layout: PipelineLayout,
    texture_bind_group_layout: BindGroupLayout,
    sampler: Sampler,

    vert_shader: Arc<ShaderModule>,
    frag_shader: Arc<ShaderModule>,

    screenspace_triangle_verts: Buffer,

    transparency_processor: compute::CutoutTransparencyCompute,
    mip_creator: compute::MipmapCompute,
    oit_renderer: oit::Oit,
    skybox_renderer: skybox::Skybox,

    command_buffers: Vec<CommandBuffer>,
    _renderdoc_capture: bool,
}

impl Renderer {
    pub async fn new(window: &Window, oit_node_count: OITNodeCount, samples: render::MSAASetting) -> Self {
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

        let vs_module = shader!(&device; opaque - vert);

        let fs_module = shader!(&device; opaque - frag);

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

        let opaque_pipeline = render::create_pipeline(&device, &pipeline_layout, &vs_module, &fs_module, samples);

        let transparency_processor = compute::CutoutTransparencyCompute::new(&device);
        let mip_creator = compute::MipmapCompute::new(&device);
        let (oit_renderer, oit_command_buffer) = oit::Oit::new(
            &device,
            &vs_module,
            &texture_bind_group_layout,
            &framebuffer,
            Vector2::new(screen_size.width, screen_size.height),
            oit_node_count,
            samples,
        );
        let skybox_renderer = skybox::Skybox::new(&device, &texture_bind_group_layout, samples);

        let screenspace_triangle_verts = screenspace::create_screen_space_verts(&device);

        let projection_matrix =
            perspective_matrix(Deg(45_f32), screen_size.width as f32 / screen_size.height as f32, 0.1);

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
            resolution: screen_size,
            samples,
            oit_node_count,
            projection_matrix,

            surface,
            device,
            queue,
            swapchain,
            framebuffer,
            depth_buffer,
            opaque_pipeline,
            pipeline_layout,
            texture_bind_group_layout,
            sampler,

            vert_shader: vs_module,
            frag_shader: fs_module,

            screenspace_triangle_verts,

            transparency_processor,
            mip_creator,
            oit_renderer,
            skybox_renderer,

            command_buffers: vec![oit_command_buffer],
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
        self.resolution = screen_size;

        self.swapchain = render::create_swapchain(&self.device, &self.surface, screen_size);

        self.oit_renderer.resize(
            &self.device,
            Vector2::new(screen_size.width, screen_size.height),
            &self.framebuffer,
            self.samples,
        );

        self.projection_matrix =
            perspective_matrix(Deg(45_f32), screen_size.width as f32 / screen_size.height as f32, 0.1);
    }

    pub fn set_samples(&mut self, samples: render::MSAASetting) {
        self.framebuffer = render::create_framebuffer(&self.device, self.resolution, samples);
        self.depth_buffer = render::create_depth_buffer(&self.device, self.resolution, samples);
        self.opaque_pipeline = render::create_pipeline(
            &self.device,
            &self.pipeline_layout,
            &self.vert_shader,
            &self.frag_shader,
            samples,
        );
        self.samples = samples;

        self.oit_renderer.set_samples(
            &self.device,
            &self.vert_shader,
            &self.framebuffer,
            Vector2::new(self.resolution.width, self.resolution.height),
            self.oit_node_count,
            samples,
        );
        self.skybox_renderer.set_samples(&self.device, samples);
    }

    pub fn set_oit_node_count(&mut self, oit_node_count: oit::OITNodeCount) {
        self.oit_renderer
            .set_node_count(&self.device, oit_node_count, self.samples);
        self.oit_node_count = oit_node_count;
    }

    pub async fn render(&mut self) {
        renderdoc! {
            let mut rd = renderdoc::RenderDoc::<renderdoc::V140>::new().expect("Could not initialize renderdoc");
            if self._renderdoc_capture {
                rd.start_frame_capture(std::ptr::null(), std::ptr::null());
            }
        }

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: Some("primary") });

        // Update skybox
        self.skybox_renderer
            .update(&self.device, &mut encoder, &self.camera, &self.projection_matrix);

        // Update objects and uniforms
        self.compute_object_distances();
        let object_references = Self::sort_objects(&self.objects);
        let matrix_buffer_opt = self.recompute_uniforms(&mut encoder, &object_references).await;

        // Retry getting a swapchain texture a couple times to smooth over spurious timeouts when tons of state changes
        let mut frame_res = self.swapchain.get_next_texture();
        for _ in 1..=3 {
            if let Ok(..) = &frame_res {
                break;
            }
            frame_res = self.swapchain.get_next_texture();
        }

        let frame = frame_res.expect("Could not get next swapchain texture");

        {
            self.oit_renderer.clear_buffers(&mut encoder);
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.framebuffer,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
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

            self.skybox_renderer.render_skybox(
                &mut rpass,
                &self.textures[&0].bind_group,
                &self.screenspace_triangle_verts,
            );

            // If se don't have a matrix buffer we have nothing to render
            if let Some(matrix_buffer) = matrix_buffer_opt.as_ref() {
                let mut current_matrix_offset = 0 as BufferAddress;

                let mut rendering_opaque = true;
                rpass.set_pipeline(&self.opaque_pipeline);
                for ((mesh_idx, texture_idx, transparent), group) in &object_references
                    .into_iter()
                    .group_by(|o| (o.mesh, o.texture, o.transparent))
                {
                    if transparent && rendering_opaque {
                        self.oit_renderer.prepare_rendering(&mut rpass);
                        rendering_opaque = false;
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
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            self.oit_renderer
                .render_transparent(&mut rpass, &self.screenspace_triangle_verts);
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
