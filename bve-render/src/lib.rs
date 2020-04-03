// +x right
// +y up
// +z away from camera

pub use crate::{object::ObjectHandle, render::MSAASetting, texture::Texture};
use bve::load::mesh::Vertex as MeshVertex;
use cgmath::{
    Array, EuclideanSpace, InnerSpace, Matrix3, Matrix4, MetricSpace, Point3, Rad, SquareMatrix, Vector2, Vector3,
    Vector4,
};
use image::{Rgba, RgbaImage};
use indexmap::map::IndexMap;
use itertools::Itertools;
use num_traits::{ToPrimitive, Zero};
use std::{cmp::Ordering, io, mem::size_of};
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};
use zerocopy::{AsBytes, FromBytes};

macro_rules! include_shader {
    (vert $name:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, ".vs.spv"));
    };
    (geo $name:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, ".gs.spv"));
    };
    (frag $name:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, ".fs.spv"));
    };
    (comp $name:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, ".cs.spv"));
    };
}

mod camera;
mod compute;
mod object;
mod render;
mod texture;

pub struct Renderer {
    objects: IndexMap<u64, object::Object>,
    object_handle_count: u64,

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
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,

    vert_shader: ShaderModule,
    frag_shader: ShaderModule,

    mip_creator: compute::MipmapCompute,

    command_buffers: Vec<CommandBuffer>,
}

impl Renderer {
    pub async fn new(window: &Window, samples: render::MSAASetting) -> Self {
        let screen_size = window.inner_size();

        let surface = Surface::create(window);

        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
            },
            BackendBit::VULKAN | BackendBit::METAL,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                extensions: Extensions {
                    anisotropic_filtering: true,
                },
                limits: Limits::default(),
            })
            .await;

        let swapchain_descriptor = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: screen_size.width,
            height: screen_size.height,
            present_mode: PresentMode::Mailbox,
        };
        let swapchain = device.create_swap_chain(&surface, &swapchain_descriptor);

        let vs = include_shader!(vert "test");
        let vs_module = device.create_shader_module(&read_spirv(io::Cursor::new(&vs[..])).unwrap());

        let fs = include_shader!(frag "test");
        let fs_module = device.create_shader_module(&read_spirv(io::Cursor::new(&fs[..])).unwrap());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
        });

        let framebuffer = render::create_framebuffer(&device, &screen_size, samples);
        let depth_buffer = render::create_depth_buffer(&device, &screen_size, samples);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        component_type: TextureComponentType::Uint,
                        dimension: TextureViewDimension::D2,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
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

        let mip_creator = compute::MipmapCompute::new(&device);

        // We need to do a couple operations on the whole pile first
        let mut renderer = Self {
            objects: IndexMap::new(),
            object_handle_count: 0,

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
            bind_group_layout,
            sampler,

            vert_shader: vs_module,
            frag_shader: fs_module,

            mip_creator,

            command_buffers: Vec::new(),
        };

        // Default texture is texture handle zero, immediately discard the handle, never to be seen again
        renderer.add_texture(&RgbaImage::from_raw(1, 1, vec![0xff, 0xff, 0xff, 0xff]).unwrap());

        renderer
    }

    pub fn set_location(&mut self, object::ObjectHandle(handle): &object::ObjectHandle, location: Vector3<f32>) {
        let object: &mut object::Object = &mut self.objects[handle];

        object.location = location;
    }

    pub fn resize(&mut self, screen_size: PhysicalSize<u32>, samples: render::MSAASetting) {
        self.framebuffer = render::create_framebuffer(&self.device, &screen_size, samples);
        self.depth_buffer = render::create_depth_buffer(&self.device, &screen_size, samples);
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
        self.screen_size = screen_size;
        self.samples = samples;

        self.swapchain = self.device.create_swap_chain(&self.surface, &SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: screen_size.width,
            height: screen_size.height,
            present_mode: PresentMode::Mailbox,
        });
    }

    pub fn get_samples(&self) -> render::MSAASetting {
        self.samples
    }

    pub fn set_samples(&mut self, samples: render::MSAASetting) {
        self.resize(self.screen_size, samples);
    }

    pub async fn render(&mut self) {
        self.recompute_uniforms().await;
        self.compute_object_distances();
        self.sort_objects();

        let frame = self.swapchain.get_next_texture().unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { todo: 0 });

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
                    clear_depth: 1.0,
                    clear_stencil: 0,
                }),
            });
            let mut opaque_ended = false;
            rpass.set_pipeline(&self.opaque_pipeline);
            for object in self.objects.values() {
                if object.transparent && !opaque_ended {
                    rpass.set_pipeline(&self.alpha_pipeline);
                    opaque_ended = true;
                }

                rpass.set_bind_group(0, &object.bind_group, &[]);
                rpass.set_vertex_buffer(0, &object.vertex_buffer, 0, 0);
                rpass.set_index_buffer(&object.index_buffer, 0, 0);
                rpass.draw_indexed(0..(object.index_count as u32), 0, 0..1);
            }
        }

        self.command_buffers.push(encoder.finish());

        self.queue.submit(&self.command_buffers);
        self.command_buffers.clear();
    }
}
