use cgmath::{Matrix4, Point3, Vector3};
use std::{collections::HashMap, io, mem::size_of};
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

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
pub struct Vertex {
    _pos: [f32; 3],
    _tex_coord: [f32; 2],
}

pub const fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObjectHandle(u64);

pub struct Object {
    vertex_buffer: Buffer,

    index_buffer: Buffer,
    index_count: u32,

    bind_group: BindGroup,

    location: Vector3<f32>,
}

pub struct Renderer {
    objects: HashMap<ObjectHandle, Object>,
    object_handle_count: u64,

    surface: Surface,
    device: Device,
    queue: Queue,
    swapchain: SwapChain,
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
}

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, //
    0.0, -1.0, 0.0, 0.0, //
    0.0, 0.0, 0.5, 0.0, //
    0.0, 0.0, 0.5, 1.0,
);

fn generate_matrix(location: Vector3<f32>, aspect_ratio: f32) -> Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);
    let mx_view: Matrix4<f32> = Matrix4::look_at(
        Point3::new(1.5, -5.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let mx_model = Matrix4::from_translation(location);
    OPENGL_TO_WGPU_MATRIX * mx_projection * mx_view * mx_model
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let window_size = window.inner_size();

        let surface = Surface::create(window);

        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::Default,
            },
            BackendBit::PRIMARY,
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
            width: window_size.width,
            height: window_size.height,
            present_mode: PresentMode::Mailbox,
        };
        let swapchain = device.create_swap_chain(&surface, &swapchain_descriptor);

        let vs = include_shader!(vert "test");
        let vs_module = device.create_shader_module(&read_spirv(io::Cursor::new(&vs[..])).unwrap());

        let fs = include_shader!(frag "test");
        let fs_module = device.create_shader_module(&read_spirv(io::Cursor::new(&fs[..])).unwrap());

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::VERTEX,
                ty: BindingType::UniformBuffer { dynamic: false },
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: IndexFormat::Uint16,
            vertex_buffers: &[VertexBufferDescriptor {
                stride: size_of::<Vertex>() as BufferAddress,
                step_mode: InputStepMode::Vertex,
                attributes: &[
                    VertexAttributeDescriptor {
                        format: VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    },
                    VertexAttributeDescriptor {
                        format: VertexFormat::Float2,
                        offset: 3 * size_of::<f32>() as BufferAddress,
                        shader_location: 1,
                    },
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            objects: HashMap::new(),
            object_handle_count: 0,

            surface,
            device,
            queue,
            swapchain,
            pipeline,
            bind_group_layout,
        }
    }

    pub fn add_object(&mut self, location: Vector3<f32>, vertices: &[Vertex], indices: &[u16]) -> ObjectHandle {
        let vertex_buffer = self
            .device
            .create_buffer_with_data(vertices.as_bytes(), BufferUsage::VERTEX);
        let index_buffer = self
            .device
            .create_buffer_with_data(indices.as_bytes(), BufferUsage::INDEX);

        let matrix = generate_matrix(location, 800.0 / 600.0);
        let matrix_ref: &[f32; 16] = matrix.as_ref();
        let matrix_buffer = self
            .device
            .create_buffer_with_data(matrix_ref.as_bytes(), BufferUsage::UNIFORM | BufferUsage::COPY_DST);

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &matrix_buffer,
                    range: 0..64,
                },
            }],
        });

        let handle = self.object_handle_count;
        self.object_handle_count += 1;
        self.objects.insert(ObjectHandle(handle), Object {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            bind_group,
            location,
        });
        ObjectHandle(handle)
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let swapchain_descriptor = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        };

        self.swapchain = self.device.create_swap_chain(&self.surface, &swapchain_descriptor);
    }

    pub fn render(&mut self) {
        let frame = self.swapchain.get_next_texture().unwrap();
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { todo: 0 });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            for object in self.objects.values() {
                rpass.set_bind_group(0, &object.bind_group, &[]);
                rpass.set_vertex_buffer(0, &object.vertex_buffer, 0, 0);
                rpass.set_index_buffer(&object.index_buffer, 0, 0);
                rpass.draw_indexed(0..(object.index_count as u32), 0, 0..1);
            }
        }

        self.queue.submit(&[encoder.finish()]);
    }
}
