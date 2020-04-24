use crate::*;
use zerocopy::AsBytes;

fn create_pipeline_pass1(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    vert: &ShaderModule,
    oit1_module: &ShaderModule,
    samples: MSAASetting,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        layout: pipeline_layout,
        vertex_stage: ProgrammableStageDescriptor {
            module: vert,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: oit1_module,
            entry_point: "main",
        }),
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Cw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Bgra8Unorm,
            color_blend: BlendDescriptor::REPLACE,
            alpha_blend: BlendDescriptor::REPLACE,
            write_mask: ColorWrite::empty(),
        }],
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: CompareFunction::GreaterEqual,
            stencil_front: StencilStateFaceDescriptor::IGNORE,
            stencil_back: StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        }),
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[
                VertexBufferDescriptor {
                    stride: size_of::<render::Vertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float3, 1 => Float3, 2 => Uchar4, 3 => Float2],
                },
                VertexBufferDescriptor {
                    stride: size_of::<Uniforms>() as BufferAddress,
                    step_mode: InputStepMode::Instance,
                    attributes: &vertex_attr_array![4 => Float4, 5 => Float4, 6 => Float4, 7 => Float4],
                },
            ],
        },
        sample_count: samples as u32,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

fn create_pipeline_pass2(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    fx_module: &ShaderModule,
    oit2_module: &ShaderModule,
    samples: MSAASetting,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        layout: pipeline_layout,
        vertex_stage: ProgrammableStageDescriptor {
            module: fx_module,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: oit2_module,
            entry_point: "main",
        }),
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Cw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Bgra8Unorm,
            color_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: CompareFunction::Always,
            stencil_front: StencilStateFaceDescriptor::IGNORE,
            stencil_back: StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        }),
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[VertexBufferDescriptor {
                stride: size_of::<ScreenSpaceVertex>() as BufferAddress,
                step_mode: InputStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float2],
            }],
        },
        sample_count: samples as u32,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

#[derive(AsBytes)]
#[repr(C)]
struct OitUniforms {
    _max_nodes: u32,
    _sample_count: u32,
}

fn create_uniform_buffer(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    head_pointer_view: &TextureView,
    node_buffer: &Buffer,
    resolution: Vector2<u32>,
    samples: MSAASetting,
) -> (Buffer, BindGroup) {
    let max_nodes = node_count(resolution);
    let uniforms = OitUniforms {
        _max_nodes: max_nodes,
        _sample_count: samples as u32,
    };
    let uniform_buffer = device.create_buffer_with_data(uniforms.as_bytes(), BufferUsage::UNIFORM);

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(&head_pointer_view),
            },
            Binding {
                binding: 1,
                resource: BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..size_of::<OitUniforms>() as BufferAddress,
                },
            },
            Binding {
                binding: 2,
                resource: BindingResource::Buffer {
                    buffer: &node_buffer,
                    range: 0..size_of_node_buffer(resolution),
                },
            },
        ],
        label: Some("oit binding"),
    });

    (uniform_buffer, bind_group)
}

fn create_oit_buffers(
    device: &Device,
    encoder: &mut CommandEncoder,
    bind_group_layout: &BindGroupLayout,
    resolution: Vector2<u32>,
    samples: MSAASetting,
) -> (TextureView, Buffer, Buffer, BindGroup) {
    let head_pointer_source_buffer = device.create_buffer_with_data(
        &vec![0xFF; (resolution.x * resolution.y * 4) as usize],
        BufferUsage::COPY_SRC,
    );

    let head_pointer_texture = device.create_texture(&TextureDescriptor {
        size: Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth: 1,
        },
        dimension: TextureDimension::D2,
        format: TextureFormat::R32Uint,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        usage: TextureUsage::COPY_DST | TextureUsage::STORAGE,
        label: Some("head pointers"),
    });

    encoder.copy_buffer_to_texture(
        BufferCopyView {
            buffer: &head_pointer_source_buffer,
            bytes_per_row: resolution.x * 4,
            rows_per_image: 0,
            offset: 0,
        },
        TextureCopyView {
            texture: &head_pointer_texture,
            origin: Origin3d { x: 0, y: 0, z: 0 },
            mip_level: 0,
            array_layer: 0,
        },
        Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth: 1,
        },
    );

    let head_pointer_view = head_pointer_texture.create_default_view();

    let node_buffer = device.create_buffer(&BufferDescriptor {
        size: size_of_node_buffer(resolution),
        usage: BufferUsage::COPY_DST | BufferUsage::STORAGE | BufferUsage::STORAGE_READ,
        label: Some("oit node buffer"),
    });

    let (uniform_buffer, bind_group) = create_uniform_buffer(
        device,
        bind_group_layout,
        &head_pointer_view,
        &node_buffer,
        resolution,
        samples,
    );

    (head_pointer_view, uniform_buffer, node_buffer, bind_group)
}

const SIZE_OF_NODE: usize = 28;

const fn node_count(resolution: Vector2<u32>) -> u32 {
    resolution.x * resolution.y * 5
}

const fn size_of_node_buffer(resolution: Vector2<u32>) -> BufferAddress {
    (node_count(resolution) as usize * SIZE_OF_NODE + 4) as BufferAddress
}

fn create_node_buffer_header() -> Vec<u8> {
    let mut vec = Vec::new();
    vec.extend_from_slice(0_u32.as_bytes());

    vec
}

#[derive(AsBytes)]
#[repr(C)]
struct ScreenSpaceVertex {
    _vertices: [f32; 2],
}

const fn vert(arg: [f32; 2]) -> ScreenSpaceVertex {
    ScreenSpaceVertex { _vertices: arg }
}

fn create_screen_space_verts(device: &Device) -> Buffer {
    let data = vec![vert([-3.0, -2.0]), vert([3.0, -3.0]), vert([0.0, 3.0])];
    device.create_buffer_with_data(data.as_bytes(), BufferUsage::VERTEX)
}

pub struct Oit {
    fx_module: ShaderModule,
    oit1_module: ShaderModule,
    oit2_module: ShaderModule,

    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,

    bind_group: BindGroup,

    head_pointer_view: TextureView,

    uniform_buffer: Buffer,

    node_source_buffer: Buffer,
    node_buffer: Buffer,

    screen_space_verts: Buffer,

    resolution: Vector2<u32>,

    pass1_pipeline: RenderPipeline,
    pass2_pipeline: RenderPipeline,
}

impl Oit {
    pub fn new(
        device: &Device,
        vert: &ShaderModule,
        opaque_bind_group_layout: &BindGroupLayout,
        resolution: Vector2<u32>,
        samples: MSAASetting,
    ) -> (Self, CommandBuffer) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("OIT texture creator"),
        });

        let fx = include_shader!(vert "fx");
        let fx_module =
            device.create_shader_module(&read_spirv(io::Cursor::new(&fx[..])).expect("Could not read shader spirv"));

        let oit1 = include_shader!(frag "oit_pass1");
        let oit1_module =
            device.create_shader_module(&read_spirv(io::Cursor::new(&oit1[..])).expect("Could not read shader spirv"));

        let oit2 = include_shader!(frag "oit_pass2");
        let oit2_module =
            device.create_shader_module(&read_spirv(io::Cursor::new(&oit2[..])).expect("Could not read shader spirv"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::StorageTexture {
                        dimension: TextureViewDimension::D2,
                        format: TextureFormat::R32Uint,
                        component_type: TextureComponentType::Uint,
                        readonly: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: false,
                    },
                },
            ],
            label: Some("oit binding"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[opaque_bind_group_layout, &bind_group_layout],
        });

        let pass1_pipeline = create_pipeline_pass1(device, &pipeline_layout, vert, &oit1_module, samples);
        let pass2_pipeline = create_pipeline_pass2(device, &pipeline_layout, &fx_module, &oit2_module, samples);

        let node_buffer_data = create_node_buffer_header();
        let node_source_buffer = device.create_buffer_with_data(&node_buffer_data, BufferUsage::COPY_SRC);

        let (head_pointer_view, uniform_buffer, node_buffer, bind_group) =
            create_oit_buffers(device, &mut encoder, &bind_group_layout, resolution, samples);

        let screen_space_verts = create_screen_space_verts(device);

        (
            Self {
                fx_module,
                oit1_module,
                oit2_module,
                bind_group_layout,
                pipeline_layout,
                bind_group,
                head_pointer_view,
                uniform_buffer,
                node_source_buffer,
                node_buffer,
                screen_space_verts,
                resolution,
                pass1_pipeline,
                pass2_pipeline,
            },
            encoder.finish(),
        )
    }

    pub fn resize(&mut self, device: &Device, resolution: Vector2<u32>, samples: MSAASetting) -> CommandBuffer {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("oit resize"),
        });

        let (head_pointer_view, max_node_buffer, node_buffer, bind_group) =
            create_oit_buffers(device, &mut encoder, &self.bind_group_layout, resolution, samples);

        self.head_pointer_view = head_pointer_view;
        self.uniform_buffer = max_node_buffer;
        self.node_buffer = node_buffer;
        self.bind_group = bind_group;
        self.resolution = resolution;

        encoder.finish()
    }

    pub fn set_samples(
        &mut self,
        device: &Device,
        vert: &ShaderModule,
        resolution: Vector2<u32>,
        samples: MSAASetting,
    ) {
        self.pass1_pipeline = create_pipeline_pass1(device, &self.pipeline_layout, vert, &self.oit1_module, samples);
        self.pass2_pipeline = create_pipeline_pass2(
            device,
            &self.pipeline_layout,
            &self.fx_module,
            &self.oit2_module,
            samples,
        );
        let (uniform_buffer, bind_group) = create_uniform_buffer(
            device,
            &self.bind_group_layout,
            &self.head_pointer_view,
            &self.node_buffer,
            resolution,
            samples,
        );
        self.uniform_buffer = uniform_buffer;
        self.bind_group = bind_group;
    }

    pub fn clear_buffers(&self, encoder: &mut CommandEncoder) {
        encoder.copy_buffer_to_buffer(&self.node_source_buffer, 0, &self.node_buffer, 0, 4);
    }

    pub fn prepare_rendering<'a>(&'a self, rpass: &mut RenderPass<'a>) {
        rpass.set_pipeline(&self.pass1_pipeline);
        rpass.set_bind_group(1, &self.bind_group, &[]);
    }

    pub fn render_transparent<'a>(&'a self, rpass: &mut RenderPass<'a>) {
        rpass.set_pipeline(&self.pass2_pipeline);
        rpass.set_bind_group(1, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, &self.screen_space_verts, 0, 0);
        rpass.draw(0..3, 0..1);
    }
}