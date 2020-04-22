use crate::*;
use cgmath::{InnerSpace, Vector2, Vector3};
use num_traits::ToPrimitive;
use std::mem::size_of;
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub _normal: [f32; 3],
    pub _color: [u8; 4],
    pub _texcoord: [f32; 2],
}

#[repr(C)]
#[derive(AsBytes)]
pub struct Uniforms {
    pub _matrix: [f32; 16],
    pub _transparent: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PipelineType {
    Normal,
    Alpha,
}

// TODO: This isn't strictly true, is this just true due to WGPU? Either way I should more elegantly support this
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum MSAASetting {
    X1 = 1,
    #[cfg(not(target_os = "macos"))]
    X2 = 2,
    #[cfg(not(target_os = "macos"))]
    X4 = 4,
    #[cfg(not(target_os = "macos"))]
    X8 = 8,
}

impl MSAASetting {
    #[cfg(not(target_os = "macos"))]
    #[must_use]
    pub fn increment(self) -> Self {
        match self {
            Self::X1 => Self::X2,
            Self::X2 => Self::X4,
            _ => Self::X8,
        }
    }

    #[cfg(target_os = "macos")]
    #[must_use]
    pub fn increment(self) -> Self {
        Self::X1
    }

    #[cfg(not(target_os = "macos"))]
    #[must_use]
    pub fn decrement(self) -> Self {
        match self {
            Self::X8 => Self::X4,
            Self::X4 => Self::X2,
            _ => Self::X1,
        }
    }

    #[cfg(target_os = "macos")]
    #[must_use]
    pub fn decrement(self) -> Self {
        Self::X1
    }
}

pub fn mip_levels(size: Vector2<impl ToPrimitive>) -> u32 {
    let float_size = size.map(|v| v.to_f32().expect("Cannot convert to f32"));
    let shortest = float_size.x.min(float_size.y);
    let mips = shortest.log2().floor();
    (mips as u32) + 1
}

pub fn enumerate_mip_levels(size: Vector2<impl ToPrimitive>) -> MipIterator {
    MipIterator {
        count: 0,
        size: size.map(|v| v.to_u32().expect("Cannot convert to u32")),
    }
}

pub struct MipIterator {
    pub count: u32,
    pub size: Vector2<u32>,
}

impl Iterator for MipIterator {
    type Item = (u32, Vector2<u32>);

    fn next(&mut self) -> Option<Self::Item> {
        self.size /= 2;
        self.count += 1;
        if self.size.x.is_zero() | self.size.y.is_zero() {
            None
        } else {
            Some((self.count, self.size))
        }
    }
}

pub fn create_pipeline(
    device: &Device,
    layout: &PipelineLayout,
    vs: &ShaderModule,
    fs: &ShaderModule,
    ty: PipelineType,
    samples: MSAASetting,
) -> RenderPipeline {
    let blend = if ty == PipelineType::Alpha {
        BlendDescriptor {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        }
    } else {
        BlendDescriptor::REPLACE
    };
    let alpha_to_coverage = ty == PipelineType::Normal;
    device.create_render_pipeline(&RenderPipelineDescriptor {
        layout,
        vertex_stage: ProgrammableStageDescriptor {
            module: vs,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: fs,
            entry_point: "main",
        }),
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Cw,
            cull_mode: CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Bgra8Unorm,
            color_blend: blend.clone(),
            alpha_blend: blend,
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::GreaterEqual,
            stencil_front: StencilStateFaceDescriptor::IGNORE,
            stencil_back: StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        }),
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[
                VertexBufferDescriptor {
                    stride: size_of::<Vertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float3, 1 => Float3, 2 => Uchar4, 3 => Float2],
                },
                VertexBufferDescriptor {
                    stride: size_of::<Uniforms>() as BufferAddress,
                    step_mode: InputStepMode::Instance,
                    attributes: &vertex_attr_array![4 => Float4, 5 => Float4, 6 => Float4, 7 => Float4, 8 => Int],
                },
            ],
        },
        sample_count: samples as u32,
        sample_mask: !0,
        alpha_to_coverage_enabled: alpha_to_coverage,
    })
}

pub fn create_depth_buffer(device: &Device, size: PhysicalSize<u32>, samples: MSAASetting) -> TextureView {
    let depth_texture = device.create_texture(&TextureDescriptor {
        size: Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: samples as u32,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        label: Some("depth buffer"),
    });
    depth_texture.create_default_view()
}

pub fn create_framebuffer(device: &Device, size: PhysicalSize<u32>, samples: MSAASetting) -> TextureView {
    let extent = Extent3d {
        width: size.width,
        height: size.height,
        depth: 1,
    };

    let tex = device.create_texture(&TextureDescriptor {
        size: extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: samples as u32,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8Unorm,
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        label: Some("framebuffer"),
    });
    tex.create_default_view()
}

pub fn create_swapchain(device: &Device, surface: &Surface, screen_size: PhysicalSize<u32>) -> SwapChain {
    device.create_swap_chain(surface, &SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Bgra8Unorm,
        width: screen_size.width,
        height: screen_size.height,
        present_mode: PresentMode::Mailbox,
    })
}

impl Renderer {
    pub fn sort_objects(&mut self) {
        self.objects.sort_by(|_, lhs, _, rhs| {
            lhs.transparent
                .cmp(&rhs.transparent)
                .then_with(|| lhs.mesh.cmp(&rhs.mesh))
                .then_with(|| lhs.texture.cmp(&rhs.texture))
        });

        // distance based sorting coming back later
        // self.objects
        //     .sort_by(|_, lhs: &object::Object, _, rhs: &object::Object| {
        //         lhs.transparent.cmp(&rhs.transparent).then_with(|| {
        //             if lhs.transparent {
        //                 // we can only get here if they are both of the same transparency,
        //                 // so I can use the transparency for one as the transparency for both
        //                 rhs.camera_distance
        //                     .partial_cmp(&lhs.camera_distance)
        //                     .unwrap_or(Ordering::Equal)
        //             } else {
        //                 lhs.camera_distance
        //                     .partial_cmp(&rhs.camera_distance)
        //                     .unwrap_or(Ordering::Equal)
        //             }
        //         })
        //     });
    }

    pub async fn recompute_uniforms(&mut self) -> Option<Buffer> {
        if self.objects.is_empty() {
            return None;
        }

        let camera_mat = self.camera.compute_matrix();

        let mut matrix_buffer_data = Vec::new();

        for ((_mesh_idx, _texture_idx, transparent), group) in
            &self.objects.values().group_by(|o| (o.mesh, o.texture, o.transparent))
        {
            for object in group {
                let matrix = object::generate_matrix(
                    &camera_mat,
                    object.location,
                    self.screen_size.width as f32 / self.screen_size.height as f32,
                );
                let transparent = transparent as u32;
                let uniforms = Uniforms {
                    _matrix: *matrix.as_ref(),
                    _transparent: transparent,
                };
                matrix_buffer_data.extend_from_slice(uniforms.as_bytes());
            }
            // alignment between groups is 256
            while matrix_buffer_data.len() & 0xFF != 0 {
                matrix_buffer_data.push(0x00_u8);
            }
        }

        let tmp_buffer = self
            .device
            .create_buffer_with_data(&matrix_buffer_data, BufferUsage::COPY_SRC);

        let matrix_buffer = self.device.create_buffer(&BufferDescriptor {
            size: matrix_buffer_data.len() as BufferAddress,
            usage: BufferUsage::COPY_DST | BufferUsage::VERTEX,
            label: Some("matrix buffer"),
        });

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("matrix updater"),
        });

        encoder.copy_buffer_to_buffer(
            &tmp_buffer,
            0,
            &matrix_buffer,
            0,
            matrix_buffer_data.len() as BufferAddress,
        );

        self.command_buffers.push(encoder.finish());

        Some(matrix_buffer)
    }

    pub fn compute_object_distances(&mut self) {
        for obj in self.objects.values_mut() {
            let mesh = &self.mesh[&obj.mesh];
            let mesh_center: Vector3<f32> = obj.location + mesh.mesh_center_offset;
            let camera_mesh_vector: Vector3<f32> = self.camera.location - mesh_center;
            let distance = camera_mesh_vector.magnitude2();
            obj.camera_distance = distance;
            // println!(
            //     "{} - {} {} {}",
            //     obj.camera_distance, obj.transparent, obj.mesh_transparent, self.textures[&obj.texture].transparent
            // );
        }
    }
}
