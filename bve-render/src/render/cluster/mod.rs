use crate::{camera::FAR_PLANE_DISTANCE, frustum::Frustum, *};
use froxels::*;

mod froxels;

const FROXELS_X: u32 = 16;
const FROXELS_Y: u32 = 16;
const FROXELS_Z: u32 = 32;
const FRUSTUM_COUNT: u32 = FROXELS_X * FROXELS_Y;
const FRUSTUM_BUFFER_SIZE: BufferAddress = (FRUSTUM_COUNT * size_of::<FrustumBytes>() as u32) as BufferAddress;
const FROXEL_COUNT: u32 = FROXELS_X * FROXELS_Y * FROXELS_Z;
const MAX_LIGHTS_PER_FROXEL: u32 = 128;
const LIGHT_LIST_SIZE: BufferAddress =
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
struct ClusterUniforms {
    _froxel_count: [u32; 4],
    _max_depth: f32,
    _padding0: [f32; 3],
}

pub struct Clustering {
    frustum_creation: FrustumCreation,

    render_bind_group_layout: BindGroupLayout,
    render_bind_group: BindGroup,
}
impl Clustering {
    pub fn new(device: &Device, encoder: &mut CommandEncoder, mx_inv_proj: Mat4, frustum: Frustum) -> Self {
        let frustum_buffer = device.create_buffer(&BufferDescriptor {
            usage: BufferUsage::COPY_DST | BufferUsage::STORAGE | BufferUsage::STORAGE_READ,
            size: FRUSTUM_BUFFER_SIZE,
            label: Some("frustum buffer"),
        });

        let cluster_uniforms = ClusterUniforms {
            _froxel_count: [FROXELS_X, FROXELS_Y, FROXELS_Z, 0],
            _max_depth: FAR_PLANE_DISTANCE,
            _padding0: Default::default(),
        };

        let cluster_uniforms_buffer = device.create_buffer_with_data(cluster_uniforms.as_bytes(), BufferUsage::UNIFORM);

        let frustum_creation = FrustumCreation::new(
            device,
            encoder,
            &frustum_buffer,
            mx_inv_proj,
            frustum,
            make_vec2(&[FROXELS_X, FROXELS_Y]),
        );

        let render_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::StorageBuffer {
                        readonly: true,
                        dynamic: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
            ],
            label: Some("clustering bind group layout"),
        });

        let render_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &render_bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &frustum_buffer,
                        range: 0..FRUSTUM_BUFFER_SIZE,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer {
                        buffer: &cluster_uniforms_buffer,
                        range: 0..size_of::<ClusterUniforms>() as BufferAddress,
                    },
                },
            ],
            label: Some("clustering bind group"),
        });

        Self {
            frustum_creation,
            render_bind_group_layout,
            render_bind_group,
        }
    }

    pub fn resize(&mut self, device: &Device, encoder: &mut CommandEncoder, mx_inv_proj: Mat4, frustum: Frustum) {
        self.frustum_creation.resize(
            device,
            encoder,
            mx_inv_proj,
            frustum,
            make_vec2(&[FROXELS_X, FROXELS_Y]),
        );
    }

    pub const fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.render_bind_group_layout
    }

    pub const fn bind_group(&self) -> &BindGroup {
        &self.render_bind_group
    }

    pub fn execute(&self, encoder: &mut CommandEncoder) {
        let mut pass = encoder.begin_compute_pass();
        self.frustum_creation.execute(&mut pass);
    }
}
