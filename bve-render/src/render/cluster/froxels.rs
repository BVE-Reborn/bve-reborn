use crate::{
    frustum::Frustum,
    render::cluster::{FrustumBytes, FROXELS_X, FROXELS_Y, FRUSTUM_BUFFER_SIZE},
    *,
};
use nalgebra_glm::UVec2;
use zerocopy::AsBytes;

#[derive(AsBytes)]
#[repr(C)]
struct FroxelUniforms {
    _inv_proj: [[f32; 4]; 4],
    _frustum: FrustumBytes,
    _frustum_count: [u32; 2],
}

pub struct FrustumCreation {
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    pipeline: ComputePipeline,
}
impl FrustumCreation {
    pub fn new(
        device: &Device,
        encoder: &mut CommandEncoder,
        frustum_buffer: &Buffer,
        mx_inv_proj: Mat4,
        frustum: Frustum,
        frustum_count: UVec2,
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
                        readonly: false,
                        dynamic: false,
                    },
                },
            ],
            label: Some("frustum creation bind group layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let shader = shader!(device; froxels - compute);

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: ProgrammableStageDescriptor {
                entry_point: "main",
                module: &*shader,
            },
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            usage: BufferUsage::COPY_DST | BufferUsage::UNIFORM,
            size: size_of::<FroxelUniforms>() as BufferAddress,
            label: Some("frustum creation uniforms"),
        });

        let uniforms = FroxelUniforms {
            _frustum: frustum.into(),
            _frustum_count: *frustum_count.as_ref(),
            _inv_proj: *mx_inv_proj.as_ref(),
        };

        let uniform_staging_buffer = device.create_buffer_with_data(uniforms.as_bytes(), BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &uniform_staging_buffer,
            0,
            &uniform_buffer,
            0,
            size_of::<FroxelUniforms>() as BufferAddress,
        );

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..(size_of::<FroxelUniforms>() as BufferAddress),
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer {
                        buffer: frustum_buffer,
                        range: 0..FRUSTUM_BUFFER_SIZE,
                    },
                },
            ],
            label: Some("frustum creation bind group"),
        });

        Self {
            uniform_buffer,
            bind_group,
            pipeline,
        }
    }

    pub fn resize(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        mx_inv_proj: Mat4,
        frustum: Frustum,
        frustum_count: UVec2,
    ) {
        let uniforms = FroxelUniforms {
            _frustum: frustum.into(),
            _frustum_count: *frustum_count.as_ref(),
            _inv_proj: *mx_inv_proj.as_ref(),
        };

        let uniform_staging_buffer = device.create_buffer_with_data(uniforms.as_bytes(), BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &uniform_staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            size_of::<FroxelUniforms>() as BufferAddress,
        );
    }

    pub fn execute(&self, encoder: &mut CommandEncoder) {
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch(FROXELS_X / 8, FROXELS_Y / 8, 1);
        }
    }
}
