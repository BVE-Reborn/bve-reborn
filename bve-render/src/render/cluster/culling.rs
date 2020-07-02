use crate::{
    camera::FAR_PLANE_DISTANCE,
    render::cluster::{FROXELS_X, FROXELS_Y, FROXELS_Z, FROXEL_COUNT},
    *,
};
use bve_conveyor::{BeltBufferId, BindGroupCache};

// TODO: Unify these with the regular uniforms? This would make bind groups sharable.
#[derive(AsBytes)]
#[repr(C)]
struct CullingUniforms {
    _cluster_count: [u32; 3],
    _light_count: u32,
    _max_depth: f32,
}

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

    pub async fn update_light_counts(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frustum_buffer: &Buffer,
        light_buffer: &AutomatedBuffer,
        light_list_buffer: &Buffer,
        light_count: u32,
    ) {
        let uniforms = CullingUniforms {
            _cluster_count: [FROXELS_X, FROXELS_Y, FROXELS_Z],
            _light_count: light_count,
            _max_depth: FAR_PLANE_DISTANCE,
        };

        self.uniform_buffer
            .write_to_buffer(device, encoder, size_of::<CullingUniforms>() as BufferAddress, |buf| {
                buf.copy_from_slice(uniforms.as_bytes())
            })
            .await;

        let bind_group_layout = &self.bind_group_layout;
        self.bind_group_key = Some(
            self.bind_group
                .create_bind_group(
                    (&self.uniform_buffer, light_buffer),
                    true,
                    move |(uniform_buffer, light_buffer)| {
                        dbg!(uniform_buffer.id, light_buffer.id);
                        device.create_bind_group(&BindGroupDescriptor {
                            layout: bind_group_layout,
                            bindings: &[
                                Binding {
                                    binding: 0,
                                    resource: BindingResource::Buffer(uniform_buffer.inner.slice(..)),
                                },
                                Binding {
                                    binding: 1,
                                    resource: BindingResource::Buffer(frustum_buffer.slice(..)),
                                },
                                Binding {
                                    binding: 2,
                                    resource: BindingResource::Buffer(light_buffer.inner.slice(..)),
                                },
                                Binding {
                                    binding: 3,
                                    resource: BindingResource::Buffer(light_list_buffer.slice(..)),
                                },
                            ],
                            label: Some("light culling bind group"),
                        })
                    },
                )
                .await,
        );
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
