use crate::{
    camera::FAR_PLANE_DISTANCE,
    render::cluster::{FROXELS_X, FROXELS_Y, FROXELS_Z, FROXEL_COUNT},
    *,
};
use wgpu_conveyor::{BeltBufferId, BindGroupCache};

// TODO: Unify these with the regular uniforms? This would make bind groups sharable.
#[repr(C)]
#[derive(Copy, Clone)]
struct CullingUniforms {
    _cluster_count: shader_types::UVec3,
    _light_count: u32,
    _max_depth: f32,
}

unsafe impl bytemuck::Zeroable for CullingUniforms {}
unsafe impl bytemuck::Pod for CullingUniforms {}

pub struct LightCulling {
    uniform_buffer: AutomatedBuffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroupCache<(BeltBufferId, BeltBufferId)>,
    bind_group_key: Option<(BeltBufferId, BeltBufferId)>,
    pipeline: ComputePipeline,
}
impl LightCulling {
    pub fn new(device: &Device, buffer_manager: &mut AutomatedBufferManager) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        readonly: true,
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        readonly: true,
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        readonly: false,
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("light culling bind group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("culling pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = shader!(device; light_culling - compute);

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("culling pipeline"),
            layout: Some(&pipeline_layout),
            compute_stage: ProgrammableStageDescriptor {
                entry_point: "main",
                module: &*shader,
            },
        });

        let uniform_buffer = buffer_manager.create_new_buffer(
            device,
            size_of::<CullingUniforms>() as BufferAddress,
            BufferUsage::UNIFORM,
            Some("culling uniform buffer"),
        );

        Self {
            uniform_buffer,
            bind_group_layout,
            bind_group: BindGroupCache::new(),
            bind_group_key: None,
            pipeline,
        }
    }

    pub fn update_light_counts(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frustum_buffer: &Buffer,
        light_buffer: &AutomatedBuffer,
        light_list_buffer: &Buffer,
        light_count: u32,
    ) {
        let uniforms = CullingUniforms {
            _cluster_count: shader_types::UVec3::from([FROXELS_X, FROXELS_Y, FROXELS_Z]),
            _light_count: light_count,
            _max_depth: FAR_PLANE_DISTANCE,
        };

        self.uniform_buffer
            .write_to_buffer(device, encoder, size_of::<CullingUniforms>() as BufferAddress, |buf| {
                buf.copy_from_slice(bytemuck::bytes_of(&uniforms))
            });

        let bind_group_layout = &self.bind_group_layout;
        self.bind_group_key = Some(self.bind_group.create_bind_group(
            (&self.uniform_buffer, light_buffer),
            true,
            move |(uniform_buffer, light_buffer)| {
                dbg!(uniform_buffer.id, light_buffer.id);
                device.create_bind_group(&BindGroupDescriptor {
                    layout: bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::Buffer(uniform_buffer.inner.slice(..)),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Buffer(frustum_buffer.slice(..)),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::Buffer(light_buffer.inner.slice(..)),
                        },
                        BindGroupEntry {
                            binding: 3,
                            resource: BindingResource::Buffer(light_list_buffer.slice(..)),
                        },
                    ],
                    label: Some("light culling bind group"),
                })
            },
        ));
    }

    pub fn execute<'a>(&'a self, pass: &mut ComputePass<'a>) {
        const ERROR_MSG: &str = "missing bind group";
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(
            0,
            self.bind_group
                .get(&self.bind_group_key.expect(ERROR_MSG))
                .expect(ERROR_MSG),
            &[],
        );
        pass.dispatch(1, FROXEL_COUNT / 64, 1);
    }
}
