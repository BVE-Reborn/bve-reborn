use crate::{camera::FAR_PLANE_DISTANCE, frustum::Frustum, *};
use bve::{runtime::LightType, UVec2};
use bve_conveyor::{BeltBufferId, BindGroupCache};
use culling::*;
use froxels::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

mod culling;
mod froxels;

const FROXELS_X: u32 = 16;
const FROXELS_Y: u32 = 16;
const FROXELS_Z: u32 = 32;
const FROXEL_COUNT: u32 = FROXELS_X * FROXELS_Y * FROXELS_Z;

const FRUSTUM_COUNT: u32 = FROXELS_X * FROXELS_Y;
const FRUSTUM_BUFFER_SIZE: BufferAddress = (FRUSTUM_COUNT * size_of::<FrustumBytes>() as u32) as BufferAddress;

const MAX_LIGHTS_PER_FROXEL: u32 = 128;
const LIGHT_LIST_BUFFER_SIZE: BufferAddress =
    (FROXEL_COUNT * MAX_LIGHTS_PER_FROXEL * size_of::<u32>() as u32) as BufferAddress;

#[derive(AsBytes)]
#[repr(C)]
struct PlaneBytes {
    _abc: [f32; 3],
    _d: f32,
}

#[derive(AsBytes)]
#[repr(C)]
struct FrustumBytes {
    _planes: [PlaneBytes; 4],
}

impl From<frustum::Frustum> for FrustumBytes {
    fn from(frustum: Frustum) -> Self {
        Self {
            _planes: [
                PlaneBytes {
                    _abc: *frustum.planes[0].abc.as_ref(),
                    _d: frustum.planes[0].d,
                },
                PlaneBytes {
                    _abc: *frustum.planes[1].abc.as_ref(),
                    _d: frustum.planes[1].d,
                },
                PlaneBytes {
                    _abc: *frustum.planes[2].abc.as_ref(),
                    _d: frustum.planes[2].d,
                },
                PlaneBytes {
                    _abc: *frustum.planes[3].abc.as_ref(),
                    _d: frustum.planes[3].d,
                },
            ],
        }
    }
}

#[derive(AsBytes)]
#[repr(C)]
struct ConeLightBytes {
    _location: [f32; 4],
    _direction: [f32; 4],
    _color: [f32; 4],
    _radius: f32,
    _angle: f32,
    _point: bool,
    _padding0: [u8; 7],
}

/// TODO: have this write directly into the buffer
fn convert_lights_to_data(input: &SlotMap<DefaultKey, RenderLightDescriptor>, mx_view: Mat4) -> Vec<ConeLightBytes> {
    input
        .values()
        .map(|light: &RenderLightDescriptor| {
            let homogeneous_location = light.location.extend(1.0);

            let transformed = mx_view * homogeneous_location;

            match &light.ty {
                LightType::Point => ConeLightBytes {
                    _location: *transformed.as_ref(),
                    _direction: [0.0; 4],
                    _color: [light.color.x(), light.color.y(), light.color.z(), 0.0],
                    _radius: light.radius,
                    _angle: 0.0,
                    _point: true,
                    _padding0: [0; 7],
                },
                LightType::Cone(cone) => ConeLightBytes {
                    _location: *transformed.as_ref(),
                    _direction: [cone.direction.x(), cone.direction.y(), cone.direction.z(), 0.0],
                    _color: [light.color.x(), light.color.y(), light.color.z(), 0.0],
                    _radius: light.radius,
                    _angle: cone.angle,
                    _point: false,
                    _padding0: [0; 7],
                },
            }
        })
        .collect_vec()
}

#[derive(AsBytes)]
#[repr(C)]
struct ClusterUniforms {
    _froxel_count: [u32; 3],
    _max_depth: f32,
}

pub struct Clustering {
    frustum_creation: FrustumCreation,
    light_culling: LightCulling,

    light_buffer: AutomatedBuffer,
    cluster_uniforms_buffer: Buffer,
    frustum_buffer: Buffer,
    light_list_buffer: Buffer,

    render_bind_group_layout: BindGroupLayout,
    render_bind_group: BindGroupCache<BeltBufferId>,
    render_bind_group_key: Option<BeltBufferId>,
}
impl Clustering {
    pub fn new(
        device: &Device,
        buffer_manager: &mut AutomatedBufferManager,
        encoder: &mut CommandEncoder,
        mx_inv_proj: Mat4,
        frustum: Frustum,
    ) -> Self {
        let frustum_buffer = device.create_buffer(&BufferDescriptor {
            usage: BufferUsage::COPY_DST | BufferUsage::STORAGE,
            size: FRUSTUM_BUFFER_SIZE,
            mapped_at_creation: false,
            label: Some("frustum buffer"),
        });

        let cluster_uniforms = ClusterUniforms {
            _froxel_count: [FROXELS_X, FROXELS_Y, FROXELS_Z],
            _max_depth: FAR_PLANE_DISTANCE,
        };

        let cluster_uniforms_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: cluster_uniforms.as_bytes(),
            usage: BufferUsage::UNIFORM,
        });

        let frustum_creation = FrustumCreation::new(
            device,
            encoder,
            &frustum_buffer,
            mx_inv_proj,
            frustum,
            UVec2::new(FROXELS_X, FROXELS_Y),
        );

        let light_list_buffer = device.create_buffer(&BufferDescriptor {
            usage: BufferUsage::STORAGE,
            size: LIGHT_LIST_BUFFER_SIZE,
            mapped_at_creation: false,
            label: Some("light list buffer"),
        });

        let light_buffer = buffer_manager.create_new_buffer(device, 0, BufferUsage::STORAGE, Some("light buffer"));

        let light_culling = LightCulling::new(device, buffer_manager);

        let render_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("cluster bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::StorageBuffer {
                        readonly: true,
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::StorageBuffer {
                        readonly: true,
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::StorageBuffer {
                        readonly: true,
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        Self {
            frustum_creation,
            light_culling,

            light_buffer,
            cluster_uniforms_buffer,
            frustum_buffer,
            light_list_buffer,

            render_bind_group_layout,
            render_bind_group: BindGroupCache::new(),
            render_bind_group_key: None,
        }
    }

    pub fn resize(&mut self, device: &Device, encoder: &mut CommandEncoder, mx_inv_proj: Mat4, frustum: Frustum) {
        self.frustum_creation
            .resize(device, encoder, mx_inv_proj, frustum, UVec2::new(FROXELS_X, FROXELS_Y));
    }

    pub const fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.render_bind_group_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        const ERROR_MESSAGE: &str = "Must call execute before bind_group";
        self.render_bind_group
            .get(&self.render_bind_group_key.expect(ERROR_MESSAGE))
            .expect(ERROR_MESSAGE)
    }

    pub async fn execute(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        lights: &SlotMap<DefaultKey, RenderLightDescriptor>,
        mx_view: Mat4,
    ) {
        let light_count = lights.len();
        let lights = convert_lights_to_data(lights, mx_view);
        let light_buffer_size = (light_count * size_of::<ConeLightBytes>()) as BufferAddress;

        self.light_buffer
            .write_to_buffer(device, encoder, light_buffer_size, |buf| {
                buf.copy_from_slice(lights.as_bytes())
            })
            .await;

        let cluster_uniforms_buffer_slice = self.cluster_uniforms_buffer.slice(..);
        let frustum_buffer_slice = self.frustum_buffer.slice(..);
        let light_list_buffer_slice = self.light_list_buffer.slice(..);
        let render_group_layout = &self.render_bind_group_layout;
        self.render_bind_group_key = Some(
            self.render_bind_group
                .create_bind_group(&self.light_buffer, true, move |light_buffer| {
                    device.create_bind_group(&BindGroupDescriptor {
                        layout: render_group_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::Buffer(cluster_uniforms_buffer_slice),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::Buffer(frustum_buffer_slice),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: BindingResource::Buffer(light_buffer.inner.slice(..)),
                            },
                            BindGroupEntry {
                                binding: 3,
                                resource: BindingResource::Buffer(light_list_buffer_slice),
                            },
                        ],
                        label: Some("clustering bind group"),
                    })
                })
                .await,
        );

        if !lights.is_empty() {
            self.light_culling
                .update_light_counts(
                    device,
                    encoder,
                    &self.frustum_buffer,
                    &self.light_buffer,
                    &self.light_list_buffer,
                    light_count as u32,
                )
                .await;

            let mut pass = encoder.begin_compute_pass();
            self.frustum_creation.execute(&mut pass);
            self.light_culling.execute(&mut pass);
        }
    }
}
