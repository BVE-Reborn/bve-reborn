use crate::*;
use std::fmt;
pub use utils::*;

pub mod cluster;
pub mod oit;
pub mod skybox;
mod utils;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DebugMode {
    None,
    Frustums,
}

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
    pub _matrix: [[f32; 4]; 4],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum MSAASetting {
    X1 = 1,
    X2 = 2,
    X4 = 4,
    X8 = 8,
}

impl MSAASetting {
    #[must_use]
    pub fn from_selection_integer(value: usize) -> Self {
        match value {
            0 => Self::X1,
            1 => Self::X2,
            2 => Self::X4,
            3 => Self::X8,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn into_selection_integer(self) -> usize {
        match self {
            Self::X1 => 0,
            Self::X2 => 1,
            Self::X4 => 2,
            Self::X8 => 3,
        }
    }

    #[must_use]
    pub fn increment(self) -> Self {
        match self {
            Self::X1 => Self::X2,
            Self::X2 => Self::X4,
            _ => Self::X8,
        }
    }

    #[must_use]
    pub fn decrement(self) -> Self {
        match self {
            Self::X8 => Self::X4,
            Self::X4 => Self::X2,
            _ => Self::X1,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Vsync {
    Enabled,
    Disabled,
}

impl Vsync {
    #[must_use]
    pub fn from_selection_boolean(value: bool) -> Self {
        match value {
            false => Self::Disabled,
            true => Self::Enabled,
        }
    }

    #[must_use]
    pub fn into_selection_boolean(self) -> bool {
        match self {
            Self::Enabled => true,
            Self::Disabled => false,
        }
    }
}

impl fmt::Display for Vsync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enabled => write!(f, "Enabled"),
            Self::Disabled => write!(f, "Disabled"),
        }
    }
}

impl Renderer {
    pub async fn render(&mut self, imgui_frame_opt: Option<imgui::Ui<'_>>) -> statistics::RendererStatistics {
        renderdoc! {
            let mut rd = renderdoc::RenderDoc::<renderdoc::V140>::new().expect("Could not initialize renderdoc");
            if self._renderdoc_capture {
                info!("Starting renderdoc capture");
                rd.start_frame_capture(std::ptr::null(), std::ptr::null());
            }
        }

        let mut stats = statistics::RendererStatistics::default();
        stats.objects = self.objects.len();
        stats.meshes = self.mesh.len();
        stats.textures = self.textures.len();

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: Some("primary") });

        let ts_start = Instant::now();
        // Update skybox
        self.skybox_renderer
            .update(&self.device, &mut encoder, &self.camera, &self.projection_matrix);
        let ts_skybox = create_timestamp(&mut stats.compute_skybox_update_time, ts_start);

        // Update objects and uniforms
        self.compute_object_distances();
        let ts_obj_distance = create_timestamp(&mut stats.compute_object_distance_time, ts_skybox);

        let object_references = self.objects.values().collect_vec();
        let ts_collect = create_timestamp(&mut stats.collect_object_refs_time, ts_obj_distance);

        let object_references =
            self.frustum_culling(self.projection_matrix * self.camera.compute_matrix(), object_references);
        let ts_frustum = create_timestamp(&mut stats.compute_frustum_culling_time, ts_collect);

        let object_references = Self::sort_objects(object_references);
        let ts_sorting = create_timestamp(&mut stats.compute_object_sorting_time, ts_frustum);

        let matrix_buffer_opt = self.recompute_uniforms(&mut encoder, &object_references).await;
        let ts_uniforms = create_timestamp(&mut stats.compute_uniforms_time, ts_sorting);

        // Retry getting a swapchain texture a couple times to smooth over spurious timeouts when tons of state changes
        let mut frame_res = self.swapchain.get_next_texture();
        for _ in 1..=3 {
            if let Ok(..) = &frame_res {
                break;
            }
            error!("Dropping frame");
            frame_res = self.swapchain.get_next_texture();
        }

        let frame = frame_res.expect("Could not get next swapchain texture");

        self.cluster_renderer.execute(&mut encoder);

        {
            self.oit_renderer.clear_buffers(&mut encoder);
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.framebuffer,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_buffer,
                    depth_load_op: LoadOp::Clear,
                    depth_store_op: StoreOp::Store,
                    stencil_load_op: LoadOp::Clear,
                    stencil_store_op: StoreOp::Store,
                    clear_depth: 1.0,
                    clear_stencil: 0,
                }),
            });

            // If se don't have a matrix buffer we have nothing to render
            if let Some(matrix_buffer) = matrix_buffer_opt.as_ref() {
                let mut current_matrix_offset = 0 as BufferAddress;

                let mut rendering_opaque = true;
                rpass.set_pipeline(&self.opaque_pipeline);
                rpass.set_bind_group(1, self.cluster_renderer.bind_group(), &[]);
                for ((mesh_idx, texture_idx, transparent), group) in &object_references
                    .into_iter()
                    .group_by(|o| (o.mesh, o.texture, o.transparent))
                {
                    if transparent && rendering_opaque {
                        self.oit_renderer.prepare_rendering(&mut rpass);
                        rendering_opaque = false;
                    }

                    let mesh = &self.mesh[&mesh_idx];
                    let texture_bind = &self.textures[&texture_idx].bind_group;
                    let count = group.count();
                    let matrix_buffer_size = (count * size_of::<Uniforms>()) as BufferAddress;

                    rpass.set_bind_group(0, texture_bind, &[]);
                    rpass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                    rpass.set_vertex_buffer(1, matrix_buffer, current_matrix_offset, matrix_buffer_size);
                    rpass.set_index_buffer(&mesh.index_buffer, 0, 0);
                    rpass.draw_indexed(0..(mesh.index_count as u32), 0, 0..(count as u32));

                    current_matrix_offset += matrix_buffer_size;
                    if current_matrix_offset & 255 != 0 {
                        current_matrix_offset += 256 - (current_matrix_offset & 255)
                    }

                    // statistics
                    if transparent {
                        stats.visible_transparent_objects += count;
                        stats.transparent_draws += 1;
                    } else {
                        stats.visible_opaque_objects += count;
                        stats.opaque_draws += 1;
                    }
                }

                stats.total_visible_objects = stats.visible_transparent_objects + stats.visible_opaque_objects;
                stats.total_draws = stats.transparent_draws + stats.opaque_draws;
            }

            if let DebugMode::None = self.debug_mode {
                self.skybox_renderer.render_skybox(
                    &mut rpass,
                    &self.textures[&self.skybox_renderer.texture_id].bind_group,
                    &self.screenspace_triangle_verts,
                );
            }
        }
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Load,
                    store_op: StoreOp::Store,
                    clear_color: Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            self.oit_renderer
                .render_transparent(&mut rpass, &self.screenspace_triangle_verts);
        }

        let ts_main_render = create_timestamp(&mut stats.render_main_cpu_time, ts_uniforms);

        if let Some(imgui_frame) = imgui_frame_opt {
            self.imgui_renderer
                .render(imgui_frame.render(), &self.device, &mut encoder, &frame.view)
                .expect("Imgui rendering failed");
        }

        let ts_imgui_render = create_timestamp(&mut stats.render_imgui_cpu_time, ts_main_render);

        self.command_buffers.push(encoder.finish());

        self.queue.submit(&self.command_buffers);
        self.command_buffers.clear();

        let _ts_wgpu_time = create_timestamp(&mut stats.render_wgpu_cpu_time, ts_imgui_render);

        create_timestamp(&mut stats.total_renderer_tick_time, ts_start);

        renderdoc! {
            if self._renderdoc_capture {
                info!("Ending renderdoc capture");
                rd.end_frame_capture(std::ptr::null(), std::ptr::null());
                self._renderdoc_capture = false;
            }
        }

        stats
    }
}
