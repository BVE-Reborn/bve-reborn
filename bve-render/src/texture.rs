use crate::*;
use image::{Rgba, RgbaImage};
use log::trace;
use nalgebra_glm::make_vec2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub(crate) u64);

impl Default for TextureHandle {
    fn default() -> Self {
        Self(0)
    }
}

pub struct Texture {
    pub bind_group: BindGroup,
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
        let mip_levels = render::mip_levels(make_vec2(&[image.width(), image.height()]));
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

        texture_descriptor.mip_level_count = mip_levels;
        let filtered_texture = self.device.create_texture(&texture_descriptor);
        let dimensions = make_vec2(&[image.width(), image.height()]);
        self.transparency_processor.compute_transparency(
            &self.device,
            &mut encoder,
            &base_texture,
            &filtered_texture,
            dimensions,
        );

        self.mip_creator
            .compute_mipmaps(&self.device, &mut encoder, &filtered_texture, dimensions);

        let texture_view = filtered_texture.create_default_view();

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
            label: None,
        });

        self.command_buffers.push(encoder.finish());

        let handle = self.texture_handle_count;
        self.texture_handle_count += 1;

        self.textures.insert(handle, Texture {
            bind_group,
            transparent,
        });

        trace!("Adding new texture #{}", handle);
        TextureHandle(handle)
    }

    pub fn remove_texture(&mut self, TextureHandle(tex_idx): &TextureHandle) {
        let _texture = self.textures.remove(tex_idx).expect("Invalid texture handle");

        debug!("Removed texture #{}", tex_idx);
        // Texture goes out of scope
    }
}
