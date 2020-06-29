#[derive(Debug, Default)]
pub struct RendererStatistics {
    pub objects: usize,
    pub meshes: usize,
    pub textures: usize,

    pub visible_opaque_objects: usize,
    pub visible_transparent_objects: usize,
    pub total_visible_objects: usize,

    pub opaque_draws: usize,
    pub transparent_draws: usize,
    pub total_draws: usize,

    pub compute_skybox_update_time: f32,
    pub compute_object_distance_time: f32,
    pub collect_object_refs_time: f32,
    pub compute_frustum_culling_time: f32,
    pub compute_object_sorting_time: f32,
    pub compute_uniforms_time: f32,
    pub render_main_cpu_time: f32,
    pub render_imgui_cpu_time: f32,
    pub render_wgpu_cpu_time: f32,
    pub render_buffer_pump_cpu_time: f32,
    pub total_renderer_tick_time: f32,
}
