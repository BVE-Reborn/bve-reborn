use crate::*;
use bve::runtime::{spawn, Pool};
use slotmap::Key;
use std::fmt;
pub use utils::*;

pub mod blit;
pub mod cluster;
pub mod skybox;
mod utils;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub _normal: [f32; 3],
    pub _color: [u8; 4],
    pub _texcoord: [f32; 2],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UniformVerts {
    pub _model_view_proj: shader_types::Mat4,
    pub _model_view: shader_types::Mat4,
    pub _inv_trans_model_view: shader_types::Mat4,
}

unsafe impl bytemuck::Zeroable for UniformVerts {}
unsafe impl bytemuck::Pod for UniformVerts {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DebugMode {
    None,
    Normals,
    Frustums,
    FrustumAddressing,
    LightCount,
}

impl DebugMode {
    #[must_use]
    pub fn from_selection_integer(value: usize) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Normals,
            2 => Self::Frustums,
            3 => Self::FrustumAddressing,
            4 => Self::LightCount,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn into_selection_integer(self) -> usize {
        match self {
            Self::None => 0,
            Self::Normals => 1,
            Self::Frustums => 2,
            Self::FrustumAddressing => 3,
            Self::LightCount => 4,
        }
    }
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
    pub fn render(&mut self, imgui_frame_opt: Option<imgui::Ui<'_>>) -> statistics::RendererStatistics {
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

        Self::recompute_uniforms(
            &self.device,
            self.projection_matrix,
            self.camera.compute_matrix(),
            &mut self.matrix_buffer,
            &mut encoder,
            &object_references,
        );

        let ts_uniforms = create_timestamp(&mut stats.compute_uniforms_time, ts_sorting);

        // Retry getting a swapchain texture a couple times to smooth over spurious timeouts when tons of state changes
        let mut frame_res = self.swapchain.get_current_frame();
        for _ in 1..=4 {
            if let Ok(..) = &frame_res {
                break;
            }
            error!("Dropping frame");
            frame_res = self.swapchain.get_current_frame();
        }

        let frame = frame_res.expect("Could not get next swapchain texture");

        self.cluster_renderer
            .execute(&self.device, &mut encoder, &self.lights, self.camera.compute_matrix());

        let matrix_buffer = if !object_references.is_empty() {
            Some(self.matrix_buffer.get_current_inner())
        } else {
            None
        };

        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: &self.framebuffer,
                resolve_target: self.unsampled_framebuffer.as_ref(),
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_buffer,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(0.0),
                    store: false,
                }),
                stencil_ops: None,
            }),
        });

        let skybox_texture_bind_group = if self.skybox_renderer.texture_id.is_null() {
            &self.textures[self.null_texture].bind_group
        } else {
            &self.textures[self.skybox_renderer.texture_id].bind_group
        };

        // If se don't have a matrix buffer we have nothing to render
        if let Some(ref matrix_buffer) = matrix_buffer {
            let mut current_matrix_offset = 0 as BufferAddress;

            let mut rendering_opaque = true;
            rpass.set_pipeline(&self.opaque_pipeline);
            rpass.set_bind_group(1, self.cluster_renderer.bind_group(), &[]);
            for ((mesh_idx, texture_idx, transparent), group) in &object_references
                .into_iter()
                .group_by(|o| (o.mesh, o.texture, o.transparent))
            {
                if transparent && rendering_opaque {
                    rendering_opaque = false;

                    self.skybox_renderer
                        .render_skybox(&mut rpass, skybox_texture_bind_group, self.debug_mode);

                    rpass.set_pipeline(&self.transparent_pipeline);
                    rpass.set_bind_group(1, self.cluster_renderer.bind_group(), &[]);
                }

                let mesh = &self.mesh[mesh_idx];
                let texture_bind = if texture_idx.is_null() {
                    &self.textures[self.null_texture].bind_group
                } else {
                    &self.textures[texture_idx].bind_group
                };
                let count = group.count();
                let matrix_buffer_size = (count * size_of::<UniformVerts>()) as BufferAddress;

                rpass.set_bind_group(0, texture_bind, &[]);
                rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                rpass.set_vertex_buffer(
                    1,
                    matrix_buffer
                        .inner
                        .slice(current_matrix_offset..(current_matrix_offset + matrix_buffer_size)),
                );
                rpass.set_index_buffer(mesh.index_buffer.slice(..));
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
        } else {
            // We don't have anything to render, so render the skybox
            self.skybox_renderer
                .render_skybox(&mut rpass, skybox_texture_bind_group, self.debug_mode);
        }

        drop(rpass);

        let mut blit_rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.framebuffer_blitter.render(&mut blit_rpass);

        drop(blit_rpass);

        let ts_main_render = create_timestamp(&mut stats.render_main_cpu_time, ts_uniforms);

        if let Some(imgui_frame) = imgui_frame_opt {
            self.imgui_renderer
                .render(
                    imgui_frame.render(),
                    &self.device,
                    &mut self.buffer_manager,
                    &mut encoder,
                    &frame.output.view,
                )
                .expect("Imgui rendering failed");
        }

        let ts_imgui_render = create_timestamp(&mut stats.render_imgui_cpu_time, ts_main_render);

        self.command_buffers.push(encoder.finish());

        self.queue.submit(self.command_buffers.drain(..));

        let ts_wgpu_time = create_timestamp(&mut stats.render_wgpu_cpu_time, ts_imgui_render);

        let futures = self.buffer_manager.pump();
        for fut in futures {
            spawn(Pool::Compute, 0, fut);
        }

        let _ts_pump_time = create_timestamp(&mut stats.render_buffer_pump_cpu_time, ts_wgpu_time);

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
