#![allow(clippy::too_many_arguments)]

use bve_conveyor::{AutomatedBuffer, AutomatedBufferManager, IdBuffer};
use imgui::{Context, DrawCmd::Elements, DrawData, DrawIdx, DrawList, DrawVert, TextureId, Textures};
use smallvec::SmallVec;
use std::{borrow::Cow, mem::size_of, num::NonZeroU64, sync::Arc};
use wgpu::*;

pub type RendererResult<T> = Result<T, RendererError>;

#[derive(Clone, Debug)]
pub enum RendererError {
    BadTexture(TextureId),
}

#[allow(dead_code)]
enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// A container for a bindable texture to be used internally.
pub struct Texture {
    bind_group: BindGroup,
}

impl Texture {
    /// Creates a new imgui texture from a wgpu texture.
    pub fn new(texture: wgpu::Texture, layout: &BindGroupLayout, device: &Device) -> Self {
        // Extract the texture view.
        let view = texture.create_view(&TextureViewDescriptor::default());

        // Create the texture sampler.
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            label: Some("imgui sampler"),
            ..Default::default()
        });

        // Create the texture bind group from the layout.
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Texture { bind_group }
    }
}

#[allow(dead_code)]
pub struct Renderer {
    pipeline: RenderPipeline,
    uniform_buffer: AutomatedBuffer,
    uniform_bind_group_layout: BindGroupLayout,
    textures: Textures<Texture>,
    texture_layout: BindGroupLayout,
    clear_color: Option<Color>,
    index_buffers: Vec<AutomatedBuffer>,
    vertex_buffers: Vec<AutomatedBuffer>,
}

impl Renderer {
    /// Create a new imgui wgpu renderer, using prebuilt spirv shaders.
    pub fn new(
        imgui: &mut Context,
        device: &Device,
        queue: &mut Queue,
        buffer_manager: &mut AutomatedBufferManager,
        format: TextureFormat,
        clear_color: Option<Color>,
    ) -> Renderer {
        let vs_bytes = include_bytes!("imgui.vert.spv");
        let fs_bytes = include_bytes!("imgui.frag.spv");

        fn compile(shader: &[u8]) -> Vec<u32> {
            let mut words = vec![];
            for bytes4 in shader.chunks(4) {
                words.push(u32::from_le_bytes([bytes4[0], bytes4[1], bytes4[2], bytes4[3]]));
            }
            words
        }

        Self::new_impl(
            imgui,
            device,
            queue,
            buffer_manager,
            format,
            clear_color,
            compile(vs_bytes),
            compile(fs_bytes),
        )
    }

    /// Create an entirely new imgui wgpu renderer.
    fn new_impl(
        imgui: &mut Context,
        device: &Device,
        queue: &mut Queue,
        buffer_manager: &mut AutomatedBufferManager,
        format: TextureFormat,
        clear_color: Option<Color>,
        vs_raw: Vec<u32>,
        fs_raw: Vec<u32>,
    ) -> Renderer {
        // Load shaders.
        let vs_module = device.create_shader_module(ShaderModuleSource::SpirV(Cow::Owned(vs_raw)));
        let fs_module = device.create_shader_module(ShaderModuleSource::SpirV(Cow::Owned(fs_raw)));

        // Create the uniform matrix buffer.
        let size = 64;
        let uniform_buffer = buffer_manager.create_new_buffer(
            device,
            size,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            Some("imgui-wgpu-uniform"),
        );

        // Create the uniform matrix buffer bind group layout.
        let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: Some(NonZeroU64::new(size).unwrap()),
                },
            }],
        });

        // Create the texture layout for further usage.
        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        component_type: TextureComponentType::Float,
                        dimension: TextureViewDimension::D2,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                },
            ],
        });

        // Create the render pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("imgui-wgpu pipeline layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_layout],
            push_constant_ranges: &[],
        });

        // Create the render pipeline.
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("imgui-wgpu pipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Cw,
                cull_mode: CullMode::None,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format,
                color_blend: BlendDescriptor {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha_blend: BlendDescriptor {
                    src_factor: BlendFactor::OneMinusDstAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: size_of::<DrawVert>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float2,
                            shader_location: 0,
                            offset: 0,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float2,
                            shader_location: 1,
                            offset: 8,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Uint,
                            shader_location: 2,
                            offset: 16,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let mut renderer = Renderer {
            pipeline,
            uniform_buffer,
            uniform_bind_group_layout,
            textures: Textures::new(),
            texture_layout,
            clear_color,
            vertex_buffers: vec![],
            index_buffers: vec![],
        };

        // Immediately load the fon texture to the GPU.
        renderer.reload_font_texture(imgui, device, queue);

        renderer
    }

    /// Render the current imgui frame.
    pub async fn render<'r>(
        &'r mut self,
        draw_data: &DrawData,
        device: &Device,
        buffer_manager: &mut AutomatedBufferManager,
        encoder: &'r mut CommandEncoder,
        view: &TextureView,
    ) -> RendererResult<()> {
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];

        // If the render area is <= 0, exit here and now.
        if !(fb_width > 0.0 && fb_height > 0.0) {
            return Ok(());
        }

        let width = draw_data.display_size[0];
        let height = draw_data.display_size[1];

        // Create and update the transform matrix for the current frame.
        // This is required to adapt to vulkan coordinates.
        // let matrix = [
        //     [2.0 / width, 0.0, 0.0, 0.0],
        //     [0.0, 2.0 / height as f32, 0.0, 0.0],
        //     [0.0, 0.0, -1.0, 0.0],
        //     [-1.0, -1.0, 0.0, 1.0],
        // ];
        let matrix = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];
        self.update_uniform_buffer(device, encoder, matrix).await;

        // Create the uniform matrix buffer bind group.
        let uniform_buffer = self.uniform_buffer.get_current_inner().await;
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(uniform_buffer.inner.slice(..)),
            }],
        });

        // Update vertex and index buffers
        for (idx, draw_list) in draw_data.draw_lists().enumerate() {
            let imgui_vert_buffer = draw_list.vtx_buffer();
            let imgui_index_buffer = draw_list.idx_buffer();
            let (vert, index) = match (self.vertex_buffers.get_mut(idx), self.index_buffers.get_mut(idx)) {
                (Some(vert), Some(index)) => (vert, index),
                (None, None) => {
                    let vert_length = (imgui_vert_buffer.len() * size_of::<DrawVert>()) as BufferAddress;
                    let index_length = (imgui_index_buffer.len() * size_of::<DrawIdx>()) as BufferAddress;
                    self.vertex_buffers.push(buffer_manager.create_new_buffer(
                        device,
                        vert_length,
                        BufferUsage::VERTEX,
                        Some(format!("imgui vert buffer number {}", idx)),
                    ));
                    self.index_buffers.push(buffer_manager.create_new_buffer(
                        device,
                        index_length,
                        BufferUsage::INDEX,
                        Some(format!("imgui vert buffer number {}", idx)),
                    ));
                    (&mut self.vertex_buffers[idx], &mut self.index_buffers[idx])
                }
                _ => unreachable!("Lengths of Vertex and Index Buffers should be in sync"),
            };
            Self::upload_vertex_buffer(device, encoder, vert, imgui_vert_buffer).await;
            Self::upload_index_buffer(device, encoder, index, imgui_index_buffer).await;
        }

        let mut vertex_buffers: SmallVec<[Arc<IdBuffer>; 4]> = SmallVec::new();
        for v in &self.vertex_buffers {
            vertex_buffers.push(v.get_current_inner().await);
        }
        let mut index_buffers: SmallVec<[Arc<IdBuffer>; 4]> = SmallVec::new();
        for i in &self.index_buffers {
            index_buffers.push(i.get_current_inner().await);
        }

        // Start a new renderpass and prepare it properly.
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: &view,
                resolve_target: None,
                ops: Operations {
                    load: match self.clear_color {
                        Some(color) => LoadOp::Clear(color),
                        _ => LoadOp::Load,
                    },
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &uniform_bind_group, &[]);

        // Execute all the imgui render work.
        for (draw_list_buffers_index, draw_list) in draw_data.draw_lists().enumerate() {
            self.render_draw_list(
                &mut rpass,
                &vertex_buffers[draw_list_buffers_index],
                &index_buffers[draw_list_buffers_index],
                &draw_list,
                draw_data.display_pos,
                draw_data.framebuffer_scale,
            )?;
        }

        Ok(())
    }

    /// Render a given `DrawList` from imgui onto a wgpu frame.
    fn render_draw_list<'render>(
        &'render self,
        rpass: &mut RenderPass<'render>,
        vertex_buffer: &'render Arc<IdBuffer>,
        index_buffer: &'render Arc<IdBuffer>,
        draw_list: &DrawList,
        clip_off: [f32; 2],
        clip_scale: [f32; 2],
    ) -> RendererResult<()> {
        let mut start = 0;

        // Make sure the current buffers are attached to the render pass.
        rpass.set_vertex_buffer(0, vertex_buffer.inner.slice(..));
        rpass.set_index_buffer(index_buffer.inner.slice(..));

        for cmd in draw_list.commands() {
            if let Elements { count, cmd_params } = cmd {
                let clip_rect = [
                    (cmd_params.clip_rect[0] - clip_off[0]) * clip_scale[0],
                    (cmd_params.clip_rect[1] - clip_off[1]) * clip_scale[1],
                    (cmd_params.clip_rect[2] - clip_off[0]) * clip_scale[0],
                    (cmd_params.clip_rect[3] - clip_off[1]) * clip_scale[1],
                ];

                // Set the current texture bind group on the renderpass.
                let texture_id = cmd_params.texture_id;
                let tex = self
                    .textures
                    .get(texture_id)
                    .ok_or_else(|| RendererError::BadTexture(texture_id))?;
                rpass.set_bind_group(1, &tex.bind_group, &[]);

                // Set scissors on the renderpass.
                let scissors = (
                    clip_rect[0].max(0.0).floor() as u32,
                    clip_rect[1].max(0.0).floor() as u32,
                    (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                    (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                );
                rpass.set_scissor_rect(scissors.0, scissors.1, scissors.2, scissors.3);

                // Draw the current batch of vertices with the renderpass.
                let end = start + count as u32;
                rpass.draw_indexed(start..end, 0, 0..1);
                start = end;
            }
        }
        Ok(())
    }

    /// Updates the current uniform buffer containing the transform matrix.
    async fn update_uniform_buffer(&mut self, device: &Device, encoder: &mut CommandEncoder, matrix: [[f32; 4]; 4]) {
        let data: &[u8] = bytemuck::cast_slice(&matrix);

        // Copy the new buffer to the real buffer.
        self.uniform_buffer
            .write_to_buffer(device, encoder, 64, |buf| buf.copy_from_slice(data))
            .await;
    }

    /// Upload the vertex buffer to the gPU.
    async fn upload_vertex_buffer(
        device: &Device,
        encoder: &mut CommandEncoder,
        buffer: &mut AutomatedBuffer,
        vertices: &[DrawVert],
    ) {
        let data = unsafe { as_byte_slice(&vertices) };
        buffer
            .write_to_buffer(device, encoder, data.len() as BufferAddress, |buf| {
                buf.copy_from_slice(data)
            })
            .await;
    }

    /// Upload the index buffer to the GPU.
    async fn upload_index_buffer(
        device: &Device,
        encoder: &mut CommandEncoder,
        buffer: &mut AutomatedBuffer,
        indices: &[DrawIdx],
    ) {
        let data: &[u8] = bytemuck::cast_slice(indices);
        buffer
            .write_to_buffer(device, encoder, data.len() as BufferAddress, |buf| {
                buf.copy_from_slice(data)
            })
            .await;
    }

    /// Updates the texture on the GPU corresponding to the current imgui font atlas.
    ///
    /// This has to be called after loading a font.
    pub fn reload_font_texture(&mut self, imgui: &mut Context, device: &Device, queue: &mut Queue) {
        let mut atlas = imgui.fonts();
        let handle = atlas.build_rgba32_texture();
        let font_texture_id = self.upload_texture(device, queue, &handle.data, handle.width, handle.height);

        atlas.tex_id = font_texture_id;
    }

    /// Creates and uploads a new wgpu texture made from the imgui font atlas.
    pub fn upload_texture(
        &mut self,
        device: &Device,
        queue: &mut Queue,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> TextureId {
        // Create the wgpu texture.
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });

        let bytes = data.len();
        queue.write_texture(
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
            },
            data,
            TextureDataLayout {
                offset: 0,
                bytes_per_row: bytes as u32 / height,
                rows_per_image: height,
            },
            Extent3d {
                width,
                height,
                depth: 1,
            },
        );

        let texture = Texture::new(texture, &self.texture_layout, device);
        self.textures.insert(texture)
    }
}

unsafe fn as_byte_slice<T>(slice: &[T]) -> &[u8] {
    let len = slice.len() * std::mem::size_of::<T>();
    let ptr = slice.as_ptr() as *const u8;
    std::slice::from_raw_parts(ptr, len)
}
