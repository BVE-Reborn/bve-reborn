// +x right
// +y up
// +z into camera

use bve::load::mesh::Vertex as MeshVertex;
use cgmath::{EuclideanSpace, Matrix3, Matrix4, Point3, Rad, Vector3};
use itertools::Itertools;
use num_traits::ToPrimitive;
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
    _normal: [f32; 3],
    _tex_coord: [f32; 2],
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObjectHandle(u64);

struct Object {
    vertex_buffer: Buffer,

    index_buffer: Buffer,
    index_count: u32,

    matrix_buffer: Buffer,
    bind_group: BindGroup,

    location: Vector3<f32>,
}

struct Camera {
    location: Vector3<f32>,
    /// radians
    pitch: f32,
    /// radians
    yaw: f32,
}

pub struct Renderer {
    objects: HashMap<u64, Object>,
    object_handle_count: u64,

    camera: Camera,

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

fn generate_matrix(mx_view: &Matrix4<f32>, location: Vector3<f32>, aspect_ratio: f32) -> Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 1000.0);
    let mx_model = Matrix4::from_translation(location);
    OPENGL_TO_WGPU_MATRIX * mx_projection * mx_view * mx_model
}

fn convert_mesh_verts_to_verts(mesh_verts: &[MeshVertex]) -> Vec<Vertex> {
    mesh_verts
        .iter()
        .map(|v| Vertex {
            _pos: v.position.clone().into(),
            _normal: v.normal.clone().into(),
            _tex_coord: v.coord.clone().into(),
        })
        .collect()
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let window_size = window.inner_size();

        let surface = Surface::create(window);

        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
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
                cull_mode: CullMode::Back,
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
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[VertexBufferDescriptor {
                stride: size_of::<Vertex>() as BufferAddress,
                step_mode: InputStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float2],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            objects: HashMap::new(),
            object_handle_count: 0,

            camera: Camera {
                location: Vector3::new(-6.0, 0.0, 3.0),
                pitch: 0.0,
                yaw: 0.0,
            },

            surface,
            device,
            queue,
            swapchain,
            pipeline,
            bind_group_layout,
        }
    }

    pub fn add_object(
        &mut self,
        location: Vector3<f32>,
        mesh_verts: &[MeshVertex],
        indices: &[impl ToPrimitive],
    ) -> ObjectHandle {
        let vertices = convert_mesh_verts_to_verts(mesh_verts);
        let indices = indices
            .iter()
            .map(|i| i.to_u32().expect("Index too large (>2^32)"))
            .collect_vec();

        let vertex_buffer = self
            .device
            .create_buffer_with_data(vertices.as_bytes(), BufferUsage::VERTEX);
        let index_buffer = self
            .device
            .create_buffer_with_data(indices.as_bytes(), BufferUsage::INDEX);

        let matrix = generate_matrix(&self.compute_camera_matrix(), location, 800.0 / 600.0);
        let matrix_ref: &[f32; 16] = matrix.as_ref();
        let matrix_buffer = self
            .device
            .create_buffer_with_data(matrix_ref.as_bytes(), BufferUsage::UNIFORM | BufferUsage::MAP_WRITE);

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
        self.objects.insert(handle, Object {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            bind_group,
            matrix_buffer,
            location,
        });
        ObjectHandle(handle)
    }

    pub fn set_location(&mut self, ObjectHandle(handle): &ObjectHandle, location: Vector3<f32>) -> Option<()> {
        let object = self.objects.get_mut(handle)?;

        object.location = location;

        Some(())
    }

    pub fn set_camera(&mut self, pitch: f32, yaw: f32) {
        self.camera.pitch = pitch;
        self.camera.yaw = yaw;
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

    fn compute_camera_matrix(&mut self) -> Matrix4<f32> {
        let look_offset = Matrix3::from_axis_angle(Vector3::unit_y(), Rad(self.camera.yaw))
            * Matrix3::from_axis_angle(Vector3::unit_x(), Rad(self.camera.pitch))
            * -Vector3::unit_z();

        Matrix4::look_at(
            Point3::from_vec(self.camera.location),
            Point3::from_vec(self.camera.location + look_offset),
            Vector3::unit_y(),
        )
    }

    async fn recompute_mvp(&mut self) {
        let camera_mat = self.compute_camera_matrix();
        for object in self.objects.values() {
            let mut buf = object
                .matrix_buffer
                .map_write(0, size_of::<Matrix4<f32>>() as u64)
                .await
                .expect("Could not map buffer");
            let matrix = generate_matrix(&camera_mat, object.location, 800.0 / 600.0);
            let matrix_ref: &[f32; 16] = matrix.as_ref();
            buf.as_slice().copy_from_slice(matrix_ref.as_bytes());
        }
    }

    pub async fn render(&mut self) {
        self.recompute_mvp().await;

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
                    clear_color: Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
                        a: 1.0,
                    },
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
