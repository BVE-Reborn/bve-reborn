use crate::{
    camera::FAR_PLANE_DISTANCE,
    render::cluster::{FROXELS_X, FROXELS_Y, FROXELS_Z, FROXEL_COUNT},
    *,
};

#[derive(AsBytes)]
#[repr(C)]
struct CullingUniforms {
    _cluster_count: [u32; 3],
    _light_count: u32,
    _max_depth: f32,
}

pub struct LightCulling {
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    pipeline: ComputePipeline,
}
impl LightCulling {
    pub fn new(
        device: &Device,
        _encoder: &mut CommandEncoder,
        frustum_buffer: &Buffer,
        light_buffer: &Buffer,
        light_list_buffer: &Buffer,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry::new(0, ShaderStage::COMPUTE, BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                }),
                BindGroupLayoutEntry::new(1, ShaderStage::COMPUTE, BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: true,
                    min_binding_size: None,
                }),
                BindGroupLayoutEntry::new(2, ShaderStage::COMPUTE, BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: true,
                    min_binding_size: None,
                }),
                BindGroupLayoutEntry::new(3, ShaderStage::COMPUTE, BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: false,
                    min_binding_size: None,
                }),
            ],
            label: Some("light culling bind group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let shader = shader!(device; light_culling - compute);

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: ProgrammableStageDescriptor {
                entry_point: "main",
                module: &*shader,
            },
        });

        let uniforms = CullingUniforms {
            _cluster_count: [FROXELS_X, FROXELS_Y, FROXELS_Z],
            _light_count: 0,
            _max_depth: FAR_PLANE_DISTANCE,
        };

        let uniform_buffer =
            device.create_buffer_with_data(uniforms.as_bytes(), BufferUsage::UNIFORM | BufferUsage::COPY_DST);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer(uniform_buffer.slice(..)),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer(frustum_buffer.slice(..)),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer(light_buffer.slice(..)),
                },
                Binding {
                    binding: 3,
                    resource: BindingResource::Buffer(light_list_buffer.slice(..)),
                },
            ],
            label: Some("light culling bind group"),
        });

        Self {
            uniform_buffer,
            bind_group,
            pipeline,
        }
    }

    pub fn update_light_counts(&self, device: &Device, encoder: &mut CommandEncoder, light_count: u32) {
        let uniforms = CullingUniforms {
            _cluster_count: [FROXELS_X, FROXELS_Y, FROXELS_Z],
            _light_count: light_count,
            _max_depth: FAR_PLANE_DISTANCE,
        };

        let uniform_staging_buffer = device.create_buffer_with_data(uniforms.as_bytes(), BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &uniform_staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            size_of::<CullingUniforms>() as BufferAddress,
        );
    }

    pub fn execute<'a>(&'a self, pass: &mut ComputePass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch(1, FROXEL_COUNT / 64, 1);
    }
}
