use crate::*;
use image::{Rgba, RgbaImage};
use wgpu::TextureView;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub(crate) u64);

impl Default for TextureHandle {
    fn default() -> Self {
        Self(0)
    }
}

pub struct Texture {
    pub texture_view: TextureView,
    pub transparent: bool,
}

pub fn is_texture_transparent(texture: &RgbaImage) -> bool {
    texture.pixels().any(|&Rgba([_, _, _, a])| a != 0 && a != 255)
}

impl Renderer {
    pub fn add_texture(&mut self, image: &RgbaImage) -> TextureHandle {
        renderdoc! {
            self._renderdoc_capture = true;
        };
        let transparent = is_texture_transparent(image);
        let extent = Extent3d {
            width: image.width(),
            height: image.height(),
            depth: 1,
        };
        let mip_levels = render::mip_levels(Vector2::new(image.width(), image.height()));
        let mut texture_descriptor = TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Uint,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST | TextureUsage::STORAGE,
            label: None,
        };
        let base_texture = self.device.create_texture(&texture_descriptor);
        let tmp_buf = self
            .device
            .create_buffer_with_data(image.as_ref(), BufferUsage::COPY_SRC);
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("texture copy"),
        });
        encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &tmp_buf,
                offset: 0,
                bytes_per_row: 4 * image.width(),
                rows_per_image: 0,
            },
            TextureCopyView {
                texture: &base_texture,
                mip_level: 0,
                array_layer: 0,
                origin: Origin3d::ZERO,
            },
            extent,
        );
        self.command_buffers.push(encoder.finish());

        texture_descriptor.mip_level_count = mip_levels;
        let filtered_texture = self.device.create_texture(&texture_descriptor);
        let dimensions = Vector2::new(image.width(), image.height());
        let transparent_command = self.transparency_processor.compute_transparency(
            &self.device,
            &base_texture,
            &filtered_texture,
            dimensions,
        );
        self.command_buffers.push(transparent_command);

        let mip_commands = self
            .mip_creator
            .compute_mipmaps(&self.device, &filtered_texture, dimensions);
        self.command_buffers.extend(mip_commands);

        let texture_view = filtered_texture.create_default_view();

        let handle = self.texture_handle_count;
        self.texture_handle_count += 1;

        self.textures.insert(handle, Texture {
            texture_view,
            transparent,
        });
        TextureHandle(handle)
    }

    #[must_use]
    pub const fn get_default_texture() -> TextureHandle {
        TextureHandle(0)
    }
}
