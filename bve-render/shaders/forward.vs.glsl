#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;
layout(location = 3) in vec2 texcoord;
layout(location = 4) in mat3x2 texcoord_transform;

layout(location = 0) out vec2 o_texcoord;
layout(location = 1) out vec3 o_normal;
layout(location = 2) flat out vec4 o_color;
layout(location = 3) flat out vec2 o_boundries_min;
layout(location = 4) flat out vec2 o_boundries_max;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 transform;
    int transparent;
} uniforms;

void main() {
    o_texcoord = texcoord_transform * vec3(texcoord, 1.0);
    o_boundries_min = texcoord_transform * vec3(0.0, 0.0, 1.0);
    o_boundries_max = texcoord_transform * vec3(1.0, 1.0, 1.0);
    o_normal = normal;
    o_color = color;
    gl_Position = uniforms.transform * vec4(position, 1.0);
}
