use crate::*;

fn create_blit_pipeline(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    vertex_shader: &ShaderModule,
    fragment_shader: &ShaderModule,
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
            format: TextureFormat::Bgra8UnormSrgb,
            alpha_blend: BlendDescriptor::REPLACE,
            color_blend: BlendDescriptor::REPLACE,
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

pub struct FramebufferBlitter {
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl FramebufferBlitter {
    pub fn new(device: &Device, texture: &TextureView, sampler: &Sampler) -> Self {
        let bind_group_layout = create_texture_bind_group_layout(device, TextureComponentType::Float);
        let bind_group = create_texture_bind_group(
            device,
            &bind_group_layout,
            texture,
            sampler,
            Some("framebuffer blit bind group"),
        );
        let vertex_shader = shader!(device; blit - vert);
        let fragment_shader = shader!(device; blit - frag);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("framebuffer blit pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = create_blit_pipeline(device, &pipeline_layout, &vertex_shader, &fragment_shader);

        Self {
            bind_group_layout,
            bind_group,
            pipeline,
        }
    }

    pub fn set_samples(&mut self, device: &Device, texture: &TextureView, sampler: &Sampler) {
        self.resize(device, texture, sampler)
    }

    pub fn resize(&mut self, device: &Device, texture: &TextureView, sampler: &Sampler) {
        self.bind_group = create_texture_bind_group(
            device,
            &self.bind_group_layout,
            texture,
            sampler,
            Some("framebuffer blit bind group"),
        );
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut RenderPass<'rpass>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}
