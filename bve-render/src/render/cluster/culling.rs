use crate::{
    camera::FAR_PLANE_DISTANCE,
    render::cluster::{
        FROXELS_X, FROXELS_Y, FROXELS_Z, FROXEL_COUNT, FRUSTUM_BUFFER_SIZE, LIGHT_BUFFER_SIZE, LIGHT_LIST_BUFFER_SIZE,
    },
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
        encoder: &mut CommandEncoder,
        frustum_buffer: &Buffer,
        light_buffer: &Buffer,
        light_list_buffer: &Buffer,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: false,
                    },
                },
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

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            usage: BufferUsage::COPY_DST | BufferUsage::UNIFORM,
            size: size_of::<CullingUniforms>() as BufferAddress,
            label: Some("light culling uniforms"),
        });

        let uniforms = CullingUniforms {
            _cluster_count: [FROXELS_X, FROXELS_Y, FROXELS_Z],
            _light_count: 0,
            _max_depth: FAR_PLANE_DISTANCE,
        };

        let uniform_staging_buffer = device.create_buffer_with_data(uniforms.as_bytes(), BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &uniform_staging_buffer,
            0,
            &uniform_buffer,
            0,
            size_of::<CullingUniforms>() as BufferAddress,
        );

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..(size_of::<CullingUniforms>() as BufferAddress),
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer {
                        buffer: frustum_buffer,
                        range: 0..FRUSTUM_BUFFER_SIZE,
                    },
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer {
                        buffer: light_buffer,
                        range: 0..LIGHT_BUFFER_SIZE,
                    },
                },
                Binding {
                    binding: 3,
                    resource: BindingResource::Buffer {
                        buffer: light_list_buffer,
                        range: 0..LIGHT_LIST_BUFFER_SIZE,
                    },
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
