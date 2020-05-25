use crate::{render::Vertex, *};
use glam::{Vec2, Vec3};
use log::trace;
use std::cmp::Ordering;

pub fn mip_levels(size: UVec2) -> u32 {
    let float_size = Vec2::new(size.x as f32, size.y as f32);
    let shortest = float_size.x().min(float_size.y());
    let mips = shortest.log2().floor();
    (mips as u32) + 1
}

pub const fn enumerate_mip_levels(size: UVec2) -> MipIterator {
    MipIterator { count: 0, size }
}

pub struct MipIterator {
    pub count: u32,
    pub size: UVec2,
}

impl Iterator for MipIterator {
    type Item = (u32, UVec2);

    fn next(&mut self) -> Option<Self::Item> {
        self.size = self.size.map(|v| v / 2);
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
    samples: MSAASetting,
) -> RenderPipeline {
    debug!("Creating opaque pipeline: samples: {}", samples as u8);
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
            format: TextureFormat::Rgba32Float,
            color_blend: BlendDescriptor::REPLACE,
            alpha_blend: BlendDescriptor::REPLACE,
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
                    stride: size_of::<UniformVerts>() as BufferAddress,
                    step_mode: InputStepMode::Instance,
                    attributes: &vertex_attr_array![4 => Float4, 5 => Float4, 6 => Float4, 7 => Float4, 8 => Float4, 9 => Float4, 10 => Float4, 11 => Float4, 12 => Float4, 13 => Float4, 14 => Float4, 15 => Float4],
                },
            ],
        },
        sample_count: samples as u32,
        sample_mask: !0,
        alpha_to_coverage_enabled: true,
    })
}

pub fn create_depth_buffer(device: &Device, size: PhysicalSize<u32>, samples: MSAASetting) -> TextureView {
    debug!(
        "Creating depth buffer: {}x{}; samples = {}",
        size.width, size.height, samples as u8
    );
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
    debug!(
        "Creating framebuffer: {}x{}; samples = {}",
        size.width, size.height, samples as u8
    );
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
        format: TextureFormat::Rgba32Float,
        usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
        label: Some("framebuffer"),
    });
    tex.create_default_view()
}

pub fn create_swapchain_descriptor(screen_size: PhysicalSize<u32>, vsync: Vsync) -> SwapChainDescriptor {
    trace!(
        "Creating swapchain descriptor: {}x{}; vsync: {}",
        screen_size.width,
        screen_size.height,
        vsync
    );
    SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Bgra8Unorm,
        width: screen_size.width,
        height: screen_size.height,
        present_mode: if vsync == Vsync::Enabled {
            PresentMode::Fifo
        } else {
            PresentMode::Mailbox
        },
    }
}

impl Renderer {
    #[must_use]
    pub fn frustum_culling<'a>(
        &self,
        mx_view_proj: Mat4,
        mut objects: Vec<&'a object::Object>,
    ) -> Vec<&'a object::Object> {
        let frustum = frustum::Frustum::from_matrix(mx_view_proj);
        objects.retain(|object| {
            let mesh: &mesh::Mesh = &self.mesh[object.mesh];
            let object_center = object.location + mesh.mesh_center_offset;
            let sphere = frustum::Sphere {
                location: object_center,
                radius: mesh.mesh_bounding_sphere_radius,
            };
            frustum.contains_sphere(sphere)
        });
        objects
    }

    #[must_use]
    pub fn sort_objects(mut objects: Vec<&object::Object>) -> Vec<&object::Object> {
        // we faff around with references as it's faster

        // Sort so groups are together
        objects.sort_by_key(|o| (o.transparent, o.mesh, o.texture));

        // Split into the groups
        let mut vector_of_groups = Vec::new();
        for ((transparent, ..), group) in &objects.into_iter().group_by(|o| (o.transparent, o.mesh, o.texture)) {
            let mut vec: Vec<&object::Object> = group.collect_vec();
            // Find average of the group's distance
            let average: f32 = vec.iter().map(|v| v.camera_distance).sum::<f32>() / vec.len() as f32;
            // Sort group by distance internally
            vec.sort_by(|o1, o2| {
                o1.camera_distance
                    .partial_cmp(&o2.camera_distance)
                    .unwrap_or(Ordering::Equal)
            });
            vector_of_groups.push((vec, transparent, average));
        }

        // Sort the groups by average distance, ensuring transparency stays together
        vector_of_groups.sort_by(|(_, transparent1, dist1), (_, transparent2, dist2)| {
            transparent1.cmp(transparent2).then_with(|| {
                if *transparent1 {
                    dist2.partial_cmp(dist1).unwrap_or(Ordering::Equal)
                } else {
                    dist1.partial_cmp(dist2).unwrap_or(Ordering::Equal)
                }
            })
        });

        vector_of_groups
            .into_iter()
            .flat_map(|(group, ..)| group.into_iter())
            .collect_vec()
    }

    pub async fn recompute_uniforms(
        &self,
        encoder: &mut CommandEncoder,
        objects: &[&object::Object],
    ) -> Option<Buffer> {
        if objects.is_empty() {
            return None;
        }

        let camera_mat = self.camera.compute_matrix();

        let mut matrix_buffer_data = Vec::new();

        for (_, group) in &objects.iter().group_by(|o| (o.mesh, o.texture, o.transparent)) {
            for object in group {
                let (mx_model_view_proj, mx_model_view, mx_inv_trans_model_view) =
                    object::generate_matrix(&self.projection_matrix, &camera_mat, object.location);
                let uniforms = UniformVerts {
                    _model_view_proj: *mx_model_view_proj.as_ref(),
                    _model_view: *mx_model_view.as_ref(),
                    _inv_trans_model_view: *mx_inv_trans_model_view.as_ref(),
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

        encoder.copy_buffer_to_buffer(
            &tmp_buffer,
            0,
            &matrix_buffer,
            0,
            matrix_buffer_data.len() as BufferAddress,
        );

        Some(matrix_buffer)
    }

    pub fn compute_object_distances(&mut self) {
        for obj in self.objects.values_mut() {
            let mesh = &self.mesh[obj.mesh];
            let mesh_center: Vec3 = obj.location + mesh.mesh_center_offset;
            let camera_mesh_vector: Vec3 = self.camera.location - mesh_center;
            let distance = camera_mesh_vector.length_squared();
            obj.camera_distance = distance;
            // println!(
            //     "{} - {} {} {}",
            //     obj.camera_distance, obj.transparent, obj.mesh_transparent, self.textures[&obj.texture].transparent
            // );
        }
    }
}
