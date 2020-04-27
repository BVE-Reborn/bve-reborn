#version 450

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

    outColor = vec4(pow(vec3(normalize(world)), vec3(1 / 2.2)), 1.0);
}
