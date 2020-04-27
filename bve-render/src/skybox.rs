use crate::*;
use zerocopy::AsBytes;

#[derive(AsBytes)]
#[repr(C)]
pub struct SkyboxVertex {
    _vertex: [f32; 3],
}

#[derive(AsBytes)]
#[repr(C)]
pub struct SkyboxUniforms {
    _mvp: [f32; 16],
    _mv: [f32; 16],
}

pub fn sb_vert(vertex: [f32; 3]) -> SkyboxVertex {
    SkyboxVertex { _vertex: vertex }
}

pub fn create_box_buffers(device: &Device) -> (Buffer, Buffer) {
    let vertices = [
        sb_vert([-1.0, -1.0, 1.0]),
        sb_vert([1.0, -1.0, 1.0]),
        sb_vert([-1.0, 1.0, 1.0]),
        sb_vert([1.0, 1.0, 1.0]),
        sb_vert([-1.0, -1.0, -1.0]),
        sb_vert([1.0, -1.0, -1.0]),
        sb_vert([-1.0, 1.0, -1.0]),
        sb_vert([1.0, 1.0, -1.0]),
    ];

    let indices = [0, 1, 2, 3, 7, 1, 5, 4, 7, 6, 2, 4, 0, 1];

    let v_buf = device.create_buffer_with_data(vertices.as_bytes(), BufferUsage::VERTEX);
    let i_buf = device.create_buffer_with_data(indices.as_bytes(), BufferUsage::INDEX);

    (v_buf, i_buf)
}

fn create_pipeline(device: &Device, pipeline_layout: &PipelineLayout, samples: MSAASetting) -> RenderPipeline {
    let vs = shader!(device; skybox - vert);
    let fs = shader!(device; skybox - frag);
    device.create_render_pipeline(&RenderPipelineDescriptor {
        layout: pipeline_layout,
        vertex_stage: ProgrammableStageDescriptor {
            module: &vs,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &fs,
            entry_point: "main",
        }),
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: PrimitiveTopology::TriangleStrip,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Bgra8Unorm,
            color_blend: BlendDescriptor::REPLACE,
            alpha_blend: BlendDescriptor::REPLACE,
            write_mask: ColorWrite::ALL,
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
            vertex_buffers: &[VertexBufferDescriptor {
                stride: size_of::<SkyboxVertex>() as BufferAddress,
                step_mode: InputStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float3],
            }],
        },
        sample_count: samples as u32,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

pub struct Skybox {
    pipeline: RenderPipeline,
    pipeline_layout: PipelineLayout,
    bind_group: BindGroup,

    uniform_buffer: Buffer,

    vertices_buffer: Buffer,
    indices_buffer: Buffer,
}
impl Skybox {
    pub fn new(device: &Device, texture_bind_group_layout: &BindGroupLayout, samples: MSAASetting) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::VERTEX,
                ty: BindingType::UniformBuffer { dynamic: false },
            }],
            label: Some("skybox"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout, texture_bind_group_layout],
        });
        let pipeline = create_pipeline(device, &pipeline_layout, samples);

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: size_of::<SkyboxUniforms>() as BufferAddress,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            label: Some("skybox uniform"),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..(size_of::<SkyboxUniforms>() as BufferAddress),
                },
            }],
            label: Some("skybox"),
        });

        let (vertices_buffer, indices_buffer) = create_box_buffers(device);

        Self {
            pipeline,
            pipeline_layout,
            bind_group,
            uniform_buffer,
            vertices_buffer,
            indices_buffer,
        }
    }

    pub fn update(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        camera: &camera::Camera,
        mx_proj: &Matrix4<f32>,
    ) {
        let mx_view = camera.compute_origin_matrix();
        // Everything is at 0.0, so we care only about V and P
        let mx_mvp = mx_proj * Matrix4::from(mx_view);
        let mx_view_bytes: &[f32; 16] = mx_proj.as_ref();
        let mx_mvp_bytes: &[f32; 16] = mx_mvp.as_ref();

        let uniform = SkyboxUniforms {
            _mvp: mx_mvp_bytes.clone(),
            _mv: mx_view_bytes.clone(),
        };

        let tmp_buffer = device.create_buffer_with_data(uniform.as_bytes(), BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &tmp_buffer,
            0,
            &self.uniform_buffer,
            0,
            size_of::<SkyboxUniforms>() as BufferAddress,
        );
    }

    pub fn set_samples(&mut self, device: &Device, samples: MSAASetting) {
        self.pipeline = create_pipeline(device, &self.pipeline_layout, samples);
    }

    pub fn render_skybox<'a>(&'a self, rpass: &mut RenderPass<'a>, texture_bind_group: &'a BindGroup) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_bind_group(1, &texture_bind_group, &[]);
        rpass.set_vertex_buffer(0, &self.vertices_buffer, 0, 0);
        rpass.set_index_buffer(&self.indices_buffer, 0, 0);
        rpass.draw_indexed(0..14, 0, 0..1);
    }
}
