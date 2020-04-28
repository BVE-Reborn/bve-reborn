#version 450

#define PI 3.1415926535626433

layout(location = 0) in vec2 clip_position;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 inv_view_proj;
};
layout(set = 1, binding = 0) uniform texture2D skybox;
layout(set = 1, binding = 1) uniform sampler skybox_sampler;

void main() {
    vec4 clip = vec4(clip_position, 1.0, 1.0);
    vec4 world = inv_view_proj * clip;
    world.xyz /= world.w;
    vec3 world_dir = normalize(vec3(world));

    // both [0, tau]
    // normally atan is [-pi, pi] around +z, but we want it [0, tau] around +z, so we need to flip Z and offset it by PI.
    // The normal rotation direction is counter clockwise -pi -> pi, but we need clockwise, so flip the resulting sign to make it pi -> -pi.
    float inv_yaw = -atan(world_dir.x, -world_dir.z) + PI;
    float pitch = asin(world_dir.y) * 2 + PI;

    outColor = vec4(vec2(yaw / (PI * 2), 0.0), 0.0, 1.0);
}
