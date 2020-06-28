use crate::{screenspace::ScreenSpaceVertex, *};
use bve_conveyor::AutomatedBuffer;
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
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Rgba32Float,
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
    bind_group: Option<BindGroup>,
    bind_group_layout: BindGroupLayout,

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
            bindings: &[BindGroupLayoutEntry::new(
                0,
                ShaderStage::FRAGMENT,
                BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
            )],
            label: Some("skybox"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout, texture_bind_group_layout],
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
            bind_group: None,
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

        dbg!(self.uniform_buffer.stats().await);

        self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer(self.uniform_buffer.get_current_inner().await.inner.slice(..)),
            }],
            label: Some("skybox"),
        }));
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
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(
            0,
            self.bind_group.as_ref().expect("update not called before render"),
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
