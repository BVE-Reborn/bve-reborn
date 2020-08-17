use log::debug;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsage, Device,
};
use zerocopy::AsBytes;

#[derive(AsBytes)]
#[repr(C)]
pub struct ScreenSpaceVertex {
    _vertices: [f32; 2],
}

const fn vert(arg: [f32; 2]) -> ScreenSpaceVertex {
    ScreenSpaceVertex { _vertices: arg }
}

pub fn create_screen_space_verts(device: &Device) -> Buffer {
    debug!("Creating screen space triangle vertex buffer");
    let data = [vert([-3.0, -3.0]), vert([3.0, -3.0]), vert([0.0, 3.0])];
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some("screen space triangle vertex"),
        usage: BufferUsage::VERTEX,
        contents: data.as_bytes(),
    })
}
