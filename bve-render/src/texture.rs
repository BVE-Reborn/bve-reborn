use crate::*;
use image::RgbaImage;
use wgpu::TextureView;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub(crate) u64);

pub struct Texture {
    pub texture_view: TextureView,
}

impl Renderer {
    pub fn add_texture(&mut self, image: &RgbaImage) -> TextureHandle {
        let extent = Extent3d {
            width: image.width(),
            height: image.height(),
            depth: 1,
        };
        let mip_levels = render::mip_levels(Vector2::new(image.width(), image.height()));
        let texture_descriptor = TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: mip_levels,
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

        let filtered_texture = self.device.create_texture(&texture_descriptor);
        let dimensions = Vector2::new(image.width(), image.height());
        let transparent_command = self.transparency_processor.compute_transparency(
            &self.device,
            &base_texture,
            &filtered_texture,
            dimensions,
        );
        self.command_buffers.push(transparent_command);

        let mip_command = self
            .mip_creator
            .compute_mipmaps(&self.device, &filtered_texture, dimensions);
        self.command_buffers.extend(mip_command);

        let texture_view = filtered_texture.create_default_view();

        let handle = self.texture_handle_count;
        self.texture_handle_count += 1;

        self.textures.insert(handle, Texture { texture_view });
        TextureHandle(handle)
    }

    #[must_use]
    pub const fn get_default_texture() -> TextureHandle {
        TextureHandle(0)
    }

    pub fn set_texture(
        &mut self,
        object::ObjectHandle(obj_idx): &object::ObjectHandle,
        TextureHandle(tex_idx): &TextureHandle,
    ) {
        let obj: &mut object::Object = &mut self.objects[obj_idx];
        let tex: &Texture = &self.textures[tex_idx];

        obj.texture = *tex_idx;

        obj.bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &obj.uniform_buffer,
                        range: 0..64,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&tex.texture_view),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
            label: None,
        });
    }
}
