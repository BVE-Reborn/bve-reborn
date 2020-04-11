use crate::*;
use cgmath::Vector2;

fn create_texture_compute_pipeline(device: &Device, source: &[u8]) -> (ComputePipeline, BindGroupLayout) {
    let shader_module =
        device.create_shader_module(&read_spirv(std::io::Cursor::new(source)).expect("Cannot read shader spirv"));

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        bindings: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::COMPUTE,
                ty: BindingType::StorageTexture {
                    dimension: TextureViewDimension::D2,
                    component_type: TextureComponentType::Uint,
                    format: TextureFormat::Rgba8Uint,
                    readonly: true,
                },
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStage::COMPUTE,
                ty: BindingType::StorageTexture {
                    dimension: TextureViewDimension::D2,
                    component_type: TextureComponentType::Uint,
                    format: TextureFormat::Rgba8Uint,
                    readonly: false,
                },
            },
        ],
        label: Some("compute"),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        layout: &pipeline_layout,
        compute_stage: ProgrammableStageDescriptor {
            module: &shader_module,
            entry_point: "main",
        },
    });

    (pipeline, bind_group_layout)
}

fn create_texture_compute_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    source: &TextureView,
    dest: &TextureView,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::TextureView(source),
            },
            Binding {
                binding: 1,
                resource: BindingResource::TextureView(dest),
            },
        ],
        label: None,
    })
}

pub struct MipmapCompute {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl MipmapCompute {
    pub fn new(device: &Device) -> Self {
        let shader_source = include_shader!(comp "mipmap");
        let (pipeline, bind_group_layout) = create_texture_compute_pipeline(device, shader_source);

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn compute_mipmaps(&self, device: &Device, texture: &Texture, dimensions: Vector2<u32>) -> Vec<CommandBuffer> {
        let mut buffers = Vec::new();

        for (level, dimensions) in render::enumerate_mip_levels(dimensions) {
            let parent = texture.create_view(&TextureViewDescriptor {
                dimension: TextureViewDimension::D2,
                format: TextureFormat::Rgba8Uint,
                aspect: TextureAspect::All,
                base_mip_level: level - 1,
                level_count: 1,
                base_array_layer: 0,
                array_layer_count: 1,
            });

            let child = texture.create_view(&TextureViewDescriptor {
                dimension: TextureViewDimension::D2,
                format: TextureFormat::Rgba8Uint,
                aspect: TextureAspect::All,
                base_mip_level: level,
                level_count: 1,
                base_array_layer: 0,
                array_layer_count: 1,
            });

            let bind_group = create_texture_compute_bind_group(&device, &self.bind_group_layout, &parent, &child);

            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Mipmap") });
            let mut cpass = encoder.begin_compute_pass();

            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch(dimensions.x, dimensions.y, 1);

            drop(cpass);

            buffers.push(encoder.finish());
        }

        buffers
    }
}

pub struct CutoutTransparencyCompute {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl CutoutTransparencyCompute {
    pub fn new(device: &Device) -> Self {
        let shader_source = include_shader!(comp "transparency");
        let (pipeline, bind_group_layout) = create_texture_compute_pipeline(device, shader_source);

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn compute_transparency(
        &self,
        device: &Device,
        texture: &Texture,
        texture_dst: &Texture,
        dimensions: Vector2<u32>,
    ) -> CommandBuffer {
        let source = texture.create_default_view();
        let dest = texture_dst.create_default_view();

        let bind_group = create_texture_compute_bind_group(&device, &self.bind_group_layout, &source, &dest);

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("CutoutTransparency"),
        });
        let mut cpass = encoder.begin_compute_pass();

        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch(dimensions.x, dimensions.y, 1);

        drop(cpass);

        encoder.finish()
    }
}
