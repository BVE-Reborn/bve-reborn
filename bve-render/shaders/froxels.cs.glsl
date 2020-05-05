#version 450

#include "frustum.glsl"

layout (local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 inv_proj;
    Frustum frustum;
    uvec2 frustum_count;
};
layout(set = 0, binding = 1) buffer Frustums {
    Frustum result_frustums[];
};

uint get_global_index(uvec2 global, uvec2 total) {
    return global.y * total.x +
           global.x;
}

void main() {
    vec2 lerp_start = vec2(gl_GlobalInvocationID) / vec2(frustum_count);
    vec2 lerp_end = vec2(gl_GlobalInvocationID + 1) / vec2(frustum_count);

    vec4 clip_space[4];
    // Top Left
    clip_space[0] = vec4(mix(-1.0, 1.0, lerp_start.x), mix(-1.0, 1.0, lerp_start.y), 1.0, 1.0);
    // Top Right
    clip_space[1] = vec4(mix(-1.0, 1.0, lerp_end.x), mix(-1.0, 1.0, lerp_start.y), 1.0, 1.0);
    // Bottom Left
    clip_space[2] = vec4(mix(-1.0, 1.0, lerp_start.x), mix(-1.0, 1.0, lerp_end.y), 1.0, 1.0);
    // Bottom Right
    clip_space[3] = vec4(mix(-1.0, 1.0, lerp_end.x), mix(-1.0, 1.0, lerp_end.y), 1.0, 1.0);

    vec3 view_space[4];
    for (int i = 0; i < 4; ++i) {
        vec4 view = inv_proj * clip_space[i];
        view.xyz /= view.w;
        view_space[i] = view.xyz;
    }

    vec3 eye_position = vec3(0.0);

    Frustum frustum;

    // Left
    frustum.planes[0] = compute_plane(eye_position, view_space[0], view_space[2]);
    // Right
    frustum.planes[1] = compute_plane(eye_position, view_space[3], view_space[1]);
    // Top
    frustum.planes[2] = compute_plane(eye_position, view_space[1], view_space[0]);
    // Bottom
    frustum.planes[3] = compute_plane(eye_position, view_space[2], view_space[3]);

    uint index = get_global_index(gl_GlobalInvocationID.xy, frustum_count);

    result_frustums[index] = frustum;
}
