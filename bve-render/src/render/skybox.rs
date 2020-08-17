use crate::{screenspace::ScreenSpaceVertex, *};
use bve_conveyor::{AutomatedBuffer, BeltBufferId, BindGroupCache};
use log::debug;
use zerocopy::AsBytes;

#[derive(AsBytes)]
#[repr(C)]
pub struct SkyboxUniforms {
    _inv_view_proj: [f32; 16],
    _repeats: f32,
}

fn create_pipeline(device: &Device, pipeline_layout: &PipelineLayout, samples: MSAASetting) -> RenderPipeline {
    debug!("Creating skybox pipeline: samples = {}", samples as u8);
    let vs = shader!(device; skybox - vert);
    let fs = shader!(device; skybox - frag);
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("skybox pipeline"),
        layout: Some(pipeline_layout),
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
            clamp_depth: false,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Rgba16Float,
            color_blend: BlendDescriptor::REPLACE,
            alpha_blend: BlendDescriptor::REPLACE,
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

pub struct Skybox {
    pipeline: RenderPipeline,
    pipeline_layout: PipelineLayout,
    bind_group: BindGroupCache<BeltBufferId>,
    bind_group_layout: BindGroupLayout,
    bind_group_key: Option<BeltBufferId>,

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
        let pipeline = create_pipeline(device, &pipeline_layout, samples);

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
            _inv_view_proj: *mx_inv_view_proj_bytes,
            _repeats: self.repeats,
        };

        self.uniform_buffer
            .write_to_buffer(device, encoder, size_of::<SkyboxUniforms>() as BufferAddress, |data| {
                data.copy_from_slice(uniform.as_bytes())
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
        self.pipeline = create_pipeline(device, &self.pipeline_layout, samples);
    }

    pub fn render_skybox<'a>(
        &'a self,
        rpass: &mut RenderPass<'a>,
        texture_bind_group: &'a BindGroup,
        screenspace_verts: &'a Buffer,
    ) {
        const ERROR_MSG: &str = "update not called before render";
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(
            0,
            self.bind_group
                .get(&self.bind_group_key.expect(ERROR_MSG))
                .expect(ERROR_MSG),
            &[],
        );
        rpass.set_bind_group(1, texture_bind_group, &[]);
        rpass.set_vertex_buffer(0, screenspace_verts.slice(..));
        rpass.draw(0..3, 0..1);
    }
}

impl Renderer {
    pub fn set_skybox_image(&mut self, TextureHandle(tex_id): &TextureHandle, repeats: f32) {
        self.skybox_renderer.texture_id = *tex_id;
        self.skybox_renderer.repeats = repeats;
    }
}
