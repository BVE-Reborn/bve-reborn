use crate::{screenspace::ScreenSpaceVertex, *};
use log::debug;
use zerocopy::AsBytes;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum OITNodeCount {
    Four = 4,
    Eight = 8,
    Sixteen = 16,
    ThirtyTwo = 32,
}

impl OITNodeCount {
    #[must_use]
    pub fn from_selection_integer(value: usize) -> Self {
        match value {
            0 => Self::Four,
            1 => Self::Eight,
            2 => Self::Sixteen,
            3 => Self::ThirtyTwo,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn into_selection_integer(self) -> usize {
        match self {
            Self::Four => 0,
            Self::Eight => 1,
            Self::Sixteen => 2,
            Self::ThirtyTwo => 3,
        }
    }

    #[must_use]
    pub fn increment(self) -> Self {
        match self {
            Self::Four => Self::Eight,
            Self::Eight => Self::Sixteen,
            _ => Self::ThirtyTwo,
        }
    }

    #[must_use]
    pub fn decrement(self) -> Self {
        match self {
            Self::ThirtyTwo => Self::Sixteen,
            Self::Sixteen => Self::Eight,
            _ => Self::Four,
        }
    }
}

fn create_pipeline_pass1(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    vert: &ShaderModule,
    samples: MSAASetting,
) -> RenderPipeline {
    debug!("Creating OIT pass1 pipeline: samples = {}", samples as u8);
    let oit1_module = shader!(device; oit_pass1 - frag);
    device.create_render_pipeline(&RenderPipelineDescriptor {
        layout: pipeline_layout,
        vertex_stage: ProgrammableStageDescriptor {
            module: vert,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &oit1_module,
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
            format: TextureFormat::Rgba32Float,
            color_blend: BlendDescriptor::REPLACE,
            alpha_blend: BlendDescriptor::REPLACE,
            write_mask: ColorWrite::empty(),
        }],
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: CompareFunction::LessEqual,
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
                    attributes: &vertex_attr_array![4 => Float4, 5 => Float4, 6 => Float4, 7 => Float4, 8 => Float4, 9 => Float4, 10 => Float4, 11 => Float4],
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
    node_count: OITNodeCount,
    samples: MSAASetting,
) -> RenderPipeline {
    debug!(
        "Creating OIT pass2 pipeline: node count = {}; samples = {}",
        node_count as u8, samples as u8
    );
    let fx_module = shader!(device; fx - vert);
    let oit2_module = shader!(device; oit_pass2 - frag: MAX_SAMPLES = samples as u8; MAX_NODES = node_count as u8);
    device.create_render_pipeline(&RenderPipelineDescriptor {
        layout: pipeline_layout,
        vertex_stage: ProgrammableStageDescriptor {
            module: &fx_module,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &oit2_module,
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
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[VertexBufferDescriptor {
                stride: size_of::<ScreenSpaceVertex>() as BufferAddress,
                step_mode: InputStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float2],
            }],
        },
        sample_count: 1,
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
    oit_bind_group_layout: &BindGroupLayout,
    framebuffer_bind_group_layout: &BindGroupLayout,
    head_pointer_view: &TextureView,
    node_buffer: &Buffer,
    framebuffer: &TextureView,
    framebuffer_sampler: &Sampler,
    resolution: UVec2,
    samples: MSAASetting,
) -> (Buffer, BindGroup, BindGroup) {
    debug!(
        "Creating OIT uniform buffer: {}x{}; samples = {}",
        resolution.x, resolution.y, samples as u8
    );
    let max_nodes = node_count(resolution);
    let uniforms = OitUniforms {
        _max_nodes: max_nodes,
        _sample_count: samples as u32,
    };
    let uniform_buffer = device.create_buffer_with_data(uniforms.as_bytes(), BufferUsage::UNIFORM);

    let oit_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: oit_bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(head_pointer_view),
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
                    buffer: node_buffer,
                    range: 0..size_of_node_buffer(resolution),
                },
            },
        ],
        label: Some("oit binding"),
    });

    let framebuffer_bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: framebuffer_bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(framebuffer),
            },
            Binding {
                binding: 1,
                resource: BindingResource::Sampler(framebuffer_sampler),
            },
        ],
        label: Some("framebuffer binding"),
    });

    (uniform_buffer, oit_bind_group, framebuffer_bind_group)
}

fn create_oit_buffers(
    device: &Device,
    encoder: &mut CommandEncoder,
    oit_bind_group_layout: &BindGroupLayout,
    framebuffer_bind_group_layout: &BindGroupLayout,
    framebuffer: &TextureView,
    framebuffer_sampler: &Sampler,
    resolution: UVec2,
    samples: MSAASetting,
) -> (TextureView, Buffer, Buffer, BindGroup, BindGroup) {
    debug!(
        "Creating OIT buffers: {}x{}; samples = {}",
        resolution.x, resolution.y, samples as u8
    );
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

    let (uniform_buffer, oit_bind_group, framebuffer_bind_group) = create_uniform_buffer(
        device,
        oit_bind_group_layout,
        framebuffer_bind_group_layout,
        &head_pointer_view,
        &node_buffer,
        framebuffer,
        framebuffer_sampler,
        resolution,
        samples,
    );

    (
        head_pointer_view,
        uniform_buffer,
        node_buffer,
        oit_bind_group,
        framebuffer_bind_group,
    )
}

fn create_pass2_pipeline_layout(
    device: &Device,
    oit_bind_group_layout: &BindGroupLayout,
    samples: MSAASetting,
) -> (BindGroupLayout, PipelineLayout) {
    debug!("Creating OIT pass2 pipeline layout: samples = {}", samples as u8);
    let framebuffer_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        bindings: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::SampledTexture {
                    component_type: TextureComponentType::Float,
                    dimension: TextureViewDimension::D2,
                    multisampled: samples != MSAASetting::X1,
                },
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::Sampler { comparison: false },
            },
        ],
        label: Some("framebuffer binding"),
    });
    let pass2_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        bind_group_layouts: &[oit_bind_group_layout, &framebuffer_bind_group_layout],
    });
    (framebuffer_bind_group_layout, pass2_pipeline_layout)
}

const SIZE_OF_NODE: usize = 28;

fn node_count(resolution: UVec2) -> u32 {
    resolution.x * resolution.y * 5
}

fn size_of_node_buffer(resolution: UVec2) -> BufferAddress {
    (node_count(resolution) as usize * SIZE_OF_NODE + 4) as BufferAddress
}

fn create_node_buffer_header() -> Vec<u8> {
    let mut vec = Vec::new();
    vec.extend_from_slice(0_u32.as_bytes());

    vec
}

pub struct Oit {
    oit_bind_group_layout: BindGroupLayout,
    framebuffer_bind_group_layout: BindGroupLayout,
    pass1_pipeline_layout: PipelineLayout,
    pass2_pipeline_layout: PipelineLayout,

    oit_bind_group: BindGroup,
    framebuffer_bind_group: BindGroup,

    head_pointer_view: TextureView,

    uniform_buffer: Buffer,

    node_source_buffer: Buffer,
    node_buffer: Buffer,

    framebuffer_sampler: Sampler,

    resolution: UVec2,

    pass1_pipeline: RenderPipeline,
    pass2_pipeline: RenderPipeline,
}

impl Oit {
    pub fn new(
        device: &Device,
        encoder: &mut CommandEncoder,
        vert: &ShaderModule,
        opaque_bind_group_layout: &BindGroupLayout,
        cluster_bind_group_layout: &BindGroupLayout,
        framebuffer: &TextureView,
        resolution: UVec2,
        oit_node_count: OITNodeCount,
        samples: MSAASetting,
    ) -> Self {
        let oit_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let pass1_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[
                opaque_bind_group_layout,
                cluster_bind_group_layout,
                &oit_bind_group_layout,
            ],
        });
        let (framebuffer_bind_group_layout, pass2_pipeline_layout) =
            create_pass2_pipeline_layout(device, &oit_bind_group_layout, samples);

        let pass1_pipeline = create_pipeline_pass1(device, &pass1_pipeline_layout, vert, samples);
        let pass2_pipeline = create_pipeline_pass2(device, &pass2_pipeline_layout, oit_node_count, samples);

        let node_buffer_data = create_node_buffer_header();
        let node_source_buffer = device.create_buffer_with_data(&node_buffer_data, BufferUsage::COPY_SRC);

        let framebuffer_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: CompareFunction::Never,
        });

        let (head_pointer_view, uniform_buffer, node_buffer, oit_bind_group, framebuffer_bind_group) =
            create_oit_buffers(
                device,
                encoder,
                &oit_bind_group_layout,
                &framebuffer_bind_group_layout,
                framebuffer,
                &framebuffer_sampler,
                resolution,
                samples,
            );

        Self {
            oit_bind_group_layout,
            framebuffer_bind_group_layout,
            pass1_pipeline_layout,
            pass2_pipeline_layout,
            oit_bind_group,
            framebuffer_bind_group,
            head_pointer_view,
            uniform_buffer,
            node_source_buffer,
            node_buffer,
            framebuffer_sampler,
            resolution,
            pass1_pipeline,
            pass2_pipeline,
        }
    }

    pub fn resize(
        &mut self,
        device: &Device,
        resolution: UVec2,
        framebuffer: &TextureView,
        samples: MSAASetting,
    ) -> CommandBuffer {
        debug!(
            "OIT Resize: {}x{}; samples = {}",
            resolution.x, resolution.y, samples as u8
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("oit resize"),
        });

        let (head_pointer_view, max_node_buffer, node_buffer, oit_bind_group, framebuffer_bind_group) =
            create_oit_buffers(
                device,
                &mut encoder,
                &self.oit_bind_group_layout,
                &self.framebuffer_bind_group_layout,
                framebuffer,
                &self.framebuffer_sampler,
                resolution,
                samples,
            );

        self.head_pointer_view = head_pointer_view;
        self.uniform_buffer = max_node_buffer;
        self.node_buffer = node_buffer;
        self.oit_bind_group = oit_bind_group;
        self.framebuffer_bind_group = framebuffer_bind_group;
        self.resolution = resolution;

        encoder.finish()
    }

    pub fn set_samples(
        &mut self,
        device: &Device,
        vert: &ShaderModule,
        framebuffer: &TextureView,
        resolution: UVec2,
        oit_node_count: OITNodeCount,
        samples: MSAASetting,
    ) {
        debug!(
            "OIT set samples: {}x{}; node count = {}; samples = {}",
            resolution.x, resolution.y, oit_node_count as u8, samples as u8
        );
        let (framebuffer_bind_group_layout, pass2_pipeline_layout) =
            create_pass2_pipeline_layout(device, &self.oit_bind_group_layout, samples);
        self.framebuffer_bind_group_layout = framebuffer_bind_group_layout;
        self.pass2_pipeline_layout = pass2_pipeline_layout;
        self.pass1_pipeline = create_pipeline_pass1(device, &self.pass1_pipeline_layout, vert, samples);
        self.pass2_pipeline = create_pipeline_pass2(device, &self.pass2_pipeline_layout, oit_node_count, samples);
        let (uniform_buffer, oit_bind_group, framebuffer_bind_group) = create_uniform_buffer(
            device,
            &self.oit_bind_group_layout,
            &self.framebuffer_bind_group_layout,
            &self.head_pointer_view,
            &self.node_buffer,
            framebuffer,
            &self.framebuffer_sampler,
            resolution,
            samples,
        );
        self.uniform_buffer = uniform_buffer;
        self.oit_bind_group = oit_bind_group;
        self.framebuffer_bind_group = framebuffer_bind_group;
    }

    pub fn set_node_count(&mut self, device: &Device, oit_node_count: OITNodeCount, samples: MSAASetting) {
        debug!(
            "OIT set node count: node count = {}; samples = {}",
            oit_node_count as u8, samples as u8
        );
        self.pass2_pipeline = create_pipeline_pass2(device, &self.pass2_pipeline_layout, oit_node_count, samples);
    }

    pub fn clear_buffers(&self, encoder: &mut CommandEncoder) {
        encoder.copy_buffer_to_buffer(&self.node_source_buffer, 0, &self.node_buffer, 0, 4);
    }

    pub fn prepare_rendering<'a>(&'a self, rpass: &mut RenderPass<'a>) {
        rpass.set_pipeline(&self.pass1_pipeline);
        rpass.set_bind_group(2, &self.oit_bind_group, &[]);
    }

    pub fn render_transparent<'a>(&'a self, rpass: &mut RenderPass<'a>, screenspace_verts: &'a Buffer) {
        rpass.set_pipeline(&self.pass2_pipeline);
        rpass.set_bind_group(0, &self.oit_bind_group, &[]);
        rpass.set_bind_group(1, &self.framebuffer_bind_group, &[]);
        rpass.set_vertex_buffer(0, screenspace_verts, 0, 0);
        rpass.draw(0..3, 0..1);
    }
}
