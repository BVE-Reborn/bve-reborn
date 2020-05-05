use crate::{frustum::Frustum, *};
use froxels::*;

mod froxels;

const FROXELS_X: u32 = 16;
const FROXELS_Y: u32 = 16;
const FROXELS_Z: u32 = 32;
const FRUSTUM_BUFFER_SIZE: BufferAddress = (FROXELS_X * FROXELS_Y * size_of::<FrustumBytes>() as u32) as BufferAddress;

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

struct Clustering {
    frustum_creation: FrustumCreation,
    frustum_buffer: Buffer,

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

        let frustum_creation = FrustumCreation::new(
            device,
            encoder,
            &frustum_buffer,
            mx_inv_proj,
            frustum,
            make_vec2(&[FROXELS_X, FROXELS_Y]),
        );

        let render_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::StorageBuffer {
                    readonly: true,
                    dynamic: false,
                },
            }],
            label: Some("clustering bind group layout"),
        });

        let render_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &render_bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &frustum_buffer,
                    range: 0..FRUSTUM_BUFFER_SIZE,
                },
            }],
            label: Some("clustering bind group"),
        });

        Self {
            frustum_creation,
            frustum_buffer,
            render_bind_group_layout,
            render_bind_group,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.render_bind_group
    }
}
