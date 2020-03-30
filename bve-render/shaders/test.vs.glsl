#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texcoord;
layout(location = 0) out vec2 o_texcoord;
layout(location = 1) out vec3 o_normal;

layout(set = 0, binding = 0) uniform Locals {
    mat4 transform;
} locals;

void main() {
    o_texcoord = texcoord;
    o_normal = normal;
    gl_Position = locals.transform * vec4(position, 1.0);
}
