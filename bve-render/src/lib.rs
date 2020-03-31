// +x right
// +y up
// +z away from camera

use bve::load::mesh::Vertex as MeshVertex;
use cgmath::{EuclideanSpace, Matrix3, Matrix4, Point3, Rad, SquareMatrix, Vector3, Vector4};
use image::RgbaImage;
use indexmap::map::IndexMap;
use itertools::Itertools;
use num_traits::ToPrimitive;
use std::{io, mem::size_of};
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

    texture: u64,

    matrix_buffer: Buffer,
    bind_group: BindGroup,

    location: Vector3<f32>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextureHandle(u64);

struct Texture {
    texture_view: TextureView,
    transparent: bool,
}

struct Camera {
    location: Vector3<f32>,
    /// radians
    pitch: f32,
    /// radians
    yaw: f32,
}

impl Camera {
    pub fn compute_matrix(&self) -> Matrix4<f32> {
        // This is pre z-inversion, so z is flipped here
        let look_offset = Matrix3::from_diagonal(Vector3::new(1.0, 1.0, -1.0))
            * Matrix3::from_axis_angle(Vector3::unit_y(), Rad(self.yaw))
            * Matrix3::from_axis_angle(Vector3::unit_x(), Rad(self.pitch))
            * Vector3::unit_z();

        Matrix4::from_diagonal(Vector4::new(1.0, 1.0, -1.0, 1.0))
            * Matrix4::look_at(
                Point3::from_vec(self.location),
                Point3::from_vec(self.location + look_offset),
                Vector3::unit_y(),
            )
    }
}

pub struct Renderer {
    objects: IndexMap<u64, Object>,
    object_handle_count: u64,

    textures: IndexMap<u64, Texture>,
    texture_handle_count: u64,

    camera: Camera,
    screen_size: PhysicalSize<u32>,

    surface: Surface,
    device: Device,
    queue: Queue,
    swapchain: SwapChain,
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,
    depth_texture_view: TextureView,

    command_buffers: Vec<CommandBuffer>,
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

fn convert_mesh_verts_to_verts(verts: Vec<MeshVertex>, mut indices: Vec<u32>) -> (Vec<Vertex>, Vec<u32>) {
    // First add the extra faces due to doubling
    let mut extra_indices = Vec::new();

    for (&i1, &i2, &i3) in indices.iter().tuples() {
        let v1_double = verts[i1 as usize].double_sided;
        let v2_double = verts[i2 as usize].double_sided;
        let v3_double = verts[i3 as usize].double_sided;

        if v1_double || v2_double || v3_double {
            extra_indices.push(i3);
            extra_indices.push(i2);
            extra_indices.push(i1);
        }
    }

    // Then convert the verts to the new format
    let out_verts = verts
        .into_iter()
        .map(|v| Vertex {
            _pos: v.position.into(),
            _normal: v.normal.into(),
            _tex_coord: v.coord.into(),
        })
        .collect_vec();

    indices.extend(extra_indices.into_iter());

    (out_verts, indices)
}

fn is_mesh_transparent(mesh: &[MeshVertex]) -> bool {
    // mesh.iter().any(|v| v.)
    unimplemented!()
}

fn is_texture_transparent(texture: &RgbaImage) -> bool {
    texture.pixels().any(|p| p.0[3] != 0 || p.0[3] != 255)
}

fn create_depth_buffer(device: &Device, size: &PhysicalSize<u32>) -> TextureView {
    let depth_texture = device.create_texture(&TextureDescriptor {
        size: Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsage::OUTPUT_ATTACHMENT,
    });
    depth_texture.create_default_view()
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let screen_size = window.inner_size();

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
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
        });

        let depth_texture_view = create_depth_buffer(&device, &screen_size);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        component_type: TextureComponentType::Float,
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

        let blend = BlendDescriptor {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        };

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
                front_face: FrontFace::Cw,
                cull_mode: CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                color_blend: blend.clone(),
                alpha_blend: blend,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil_front: StencilStateFaceDescriptor::IGNORE,
                stencil_back: StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
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

        // We need to do a couple operations on the whole pile first
        let mut renderer = Self {
            objects: IndexMap::new(),
            object_handle_count: 0,

            textures: IndexMap::new(),
            texture_handle_count: 0,

            camera: Camera {
                location: Vector3::new(-6.0, 0.0, 10.0),
                pitch: 0.0,
                yaw: 0.0,
            },
            screen_size,

            surface,
            device,
            queue,
            swapchain,
            pipeline,
            bind_group_layout,
            sampler,
            depth_texture_view,

            command_buffers: Vec::new(),
        };

        // Default texture is texture handle zero, immediately discard the handle, never to be seen again
        renderer.add_texture(&RgbaImage::from_raw(1, 1, vec![0xff, 0xff, 0xff, 0xff]).unwrap());

        renderer
    }

    pub fn add_object(
        &mut self,
        location: Vector3<f32>,
        mesh_verts: Vec<MeshVertex>,
        indices: &[impl ToPrimitive],
    ) -> ObjectHandle {
        self.add_object_texture(location, mesh_verts, indices, &TextureHandle(0))
    }

    pub fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
        mesh_verts: Vec<MeshVertex>,
        indices: &[impl ToPrimitive],
        TextureHandle(tex_idx): &TextureHandle,
    ) -> ObjectHandle {
        let tex: &Texture = &self.textures[tex_idx];

        let indices = indices
            .iter()
            .map(|i| i.to_u32().expect("Index too large (>2^32)"))
            .collect_vec();
        let (vertices, indices) = convert_mesh_verts_to_verts(mesh_verts, indices);

        let vertex_buffer = self
            .device
            .create_buffer_with_data(vertices.as_bytes(), BufferUsage::VERTEX);
        let index_buffer = self
            .device
            .create_buffer_with_data(indices.as_bytes(), BufferUsage::INDEX);

        let matrix = generate_matrix(&self.camera.compute_matrix(), location, 800.0 / 600.0);
        let matrix_ref: &[f32; 16] = matrix.as_ref();
        let matrix_buffer = self
            .device
            .create_buffer_with_data(matrix_ref.as_bytes(), BufferUsage::UNIFORM | BufferUsage::MAP_WRITE);

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &matrix_buffer,
                        range: 0..64,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&tex.texture_view),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let handle = self.object_handle_count;
        self.object_handle_count += 1;
        self.objects.insert(handle, Object {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            texture: 0,
            bind_group,
            matrix_buffer,
            location,
        });
        ObjectHandle(handle)
    }

    pub fn add_texture(&mut self, image: &RgbaImage) -> TextureHandle {
        let transparent = is_texture_transparent(image);

        let extent = Extent3d {
            width: image.width(),
            height: image.height(),
            depth: 1,
        };
        let texture = self.device.create_texture(&TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_default_view();
        let tmp_buf = self
            .device
            .create_buffer_with_data(image.as_ref(), BufferUsage::COPY_SRC);
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { todo: 0 });
        encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &tmp_buf,
                offset: 0,
                bytes_per_row: 4 * image.width(),
                rows_per_image: 0,
            },
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: Origin3d::ZERO,
            },
            extent,
        );

        self.command_buffers.push(encoder.finish());

        let handle = self.texture_handle_count;
        self.texture_handle_count += 1;

        self.textures.insert(handle, Texture {
            texture_view,
            transparent,
        });
        TextureHandle(handle)
    }

    pub fn get_default_texture() -> TextureHandle {
        TextureHandle(0)
    }

    pub fn set_location(&mut self, ObjectHandle(handle): &ObjectHandle, location: Vector3<f32>) {
        let object: &mut Object = &mut self.objects[handle];

        object.location = location;
    }

    pub fn set_texture(&mut self, ObjectHandle(obj_idx): &ObjectHandle, TextureHandle(tex_idx): &TextureHandle) {
        let obj: &mut Object = &mut self.objects[obj_idx];
        let tex: &Texture = &self.textures[tex_idx];

        obj.texture = *tex_idx;

        obj.bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &obj.matrix_buffer,
                        range: 0..64,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&tex.texture_view),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });
    }

    pub fn set_camera(&mut self, pitch: f32, yaw: f32) {
        self.camera.pitch = pitch;
        self.camera.yaw = yaw;
    }

    pub fn resize(&mut self, screen_size: PhysicalSize<u32>) {
        self.depth_texture_view = create_depth_buffer(&self.device, &screen_size);

        let swapchain_descriptor = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: screen_size.width,
            height: screen_size.height,
            present_mode: PresentMode::Mailbox,
        };

        self.swapchain = self.device.create_swap_chain(&self.surface, &swapchain_descriptor);
    }

    async fn recompute_mvp(&mut self) {
        let camera_mat = self.camera.compute_matrix();
        for object in self.objects.values() {
            let mut buf = object
                .matrix_buffer
                .map_write(0, size_of::<Matrix4<f32>>() as u64)
                .await
                .expect("Could not map buffer");
            let matrix = generate_matrix(
                &camera_mat,
                object.location,
                self.screen_size.width as f32 / self.screen_size.height as f32,
            );
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
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_load_op: LoadOp::Clear,
                    depth_store_op: StoreOp::Store,
                    stencil_load_op: LoadOp::Clear,
                    stencil_store_op: StoreOp::Store,
                    clear_depth: 1.0,
                    clear_stencil: 0,
                }),
            });
            rpass.set_pipeline(&self.pipeline);
            for object in self.objects.values() {
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
