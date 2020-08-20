use crate::*;
use bve_conveyor::{AutomatedBuffer, BeltBufferId, BindGroupCache};

fn create_skybox_pipeline(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    vertex_shader: &ShaderModule,
    fragment_shader: &ShaderModule,
    samples: MSAASetting,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("framebuffer blit bind group"),
        layout: Some(pipeline_layout),
        vertex_stage: ProgrammableStageDescriptor {
            entry_point: "main",
            module: vertex_shader,
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            entry_point: "main",
            module: fragment_shader,
        }),
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            ..Default::default()
        }),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Rgba16Float,
            alpha_blend: BlendDescriptor::REPLACE,
            color_blend: BlendDescriptor::REPLACE,
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: CompareFunction::GreaterEqual,
            stencil: StencilStateDescriptor::default(),
        }),
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[],
        },
        sample_count: samples as u32,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SkyboxUniforms {
    _inv_view_proj: shader_types::Mat4,
    _repeats: f32,
}

unsafe impl bytemuck::Zeroable for SkyboxUniforms {}
unsafe impl bytemuck::Pod for SkyboxUniforms {}

pub struct Skybox {
    pipeline: RenderPipeline,
    pipeline_layout: PipelineLayout,
    bind_group: BindGroupCache<BeltBufferId>,
    bind_group_layout: BindGroupLayout,
    bind_group_key: Option<BeltBufferId>,

    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,

    uniform_buffer: AutomatedBuffer,

    pub texture_id: DefaultKey,
    pub repeats: f32,
}
impl Skybox {
    pub fn new(
        buffer_manager: &mut AutomatedBufferManager,
        device: &Device,
        texture_bind_group_layout: &BindGroupLayout,
        samples: MSAASetting,
    ) -> Self {
        let vertex_shader = shader!(device; skybox - vert);
        let fragment_shader = shader!(device; skybox - frag);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("skybox"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("skybox pipeline layout"),
            bind_group_layouts: &[&bind_group_layout, texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = create_skybox_pipeline(device, &pipeline_layout, &vertex_shader, &fragment_shader, samples);

        let uniform_buffer = buffer_manager.create_new_buffer(
            device,
            size_of::<SkyboxUniforms>() as BufferAddress,
            BufferUsage::UNIFORM,
            Some("skybox uniform"),
        );

        Self {
            pipeline,
            pipeline_layout,
            bind_group: BindGroupCache::new(),
            bind_group_key: None,
            bind_group_layout,

            vertex_shader,
            fragment_shader,

            uniform_buffer,

            texture_id: DefaultKey::default(),
            repeats: 1.0,
        }
    }

    pub async fn update(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        camera: &camera::Camera,
        mx_proj: &Mat4,
    ) {
        let mx_view = camera.compute_origin_matrix();
        let mx_view_proj = *mx_proj * mx_view;
        let mx_inv_view_proj = mx_view_proj.inverse();
        let mx_inv_view_proj_bytes: &[f32; 16] = mx_inv_view_proj.as_ref();

        let uniform = SkyboxUniforms {
            _inv_view_proj: shader_types::Mat4::from(*mx_inv_view_proj_bytes),
            _repeats: self.repeats,
        };

        self.uniform_buffer
            .write_to_buffer(device, encoder, size_of::<SkyboxUniforms>() as BufferAddress, |data| {
                data.copy_from_slice(bytemuck::bytes_of(&uniform))
            })
            .await;

        let bind_group_layout = &self.bind_group_layout;
        self.bind_group_key = Some(
            self.bind_group
                .create_bind_group(&self.uniform_buffer, true, move |uniform_buffer| {
                    device.create_bind_group(&BindGroupDescriptor {
                        layout: bind_group_layout,
                        entries: &[BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::Buffer(uniform_buffer.inner.slice(..)),
                        }],
                        label: Some("skybox"),
                    })
                })
                .await,
        );
    }

    pub fn set_samples(&mut self, device: &Device, samples: MSAASetting) {
        self.pipeline = create_skybox_pipeline(
            device,
            &self.pipeline_layout,
            &self.vertex_shader,
            &self.fragment_shader,
            samples,
        );
    }

    pub fn render_skybox<'a>(
        &'a self,
        rpass: &mut RenderPass<'a>,
        texture_bind_group: &'a BindGroup,
        debug: DebugMode,
    ) {
        const ERROR_MSG: &str = "update not called before render";
        if debug != DebugMode::None {
            return;
        }
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(
            0,
            self.bind_group
                .get(&self.bind_group_key.expect(ERROR_MSG))
                .expect(ERROR_MSG),
            &[],
        );
        rpass.set_bind_group(1, texture_bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}

impl Renderer {
    pub fn set_skybox_image(&mut self, TextureHandle(tex_id): &TextureHandle, repeats: f32) {
        self.skybox_renderer.texture_id = *tex_id;
        self.skybox_renderer.repeats = repeats;
    }
}
