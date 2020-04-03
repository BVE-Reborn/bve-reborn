#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;
layout(location = 3) in vec2 texcoord;
layout(location = 0) out vec2 o_texcoord;
layout(location = 1) out vec3 o_normal;
layout(location = 2) flat out vec4 o_color;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 transform;
    int transparent;
} uniforms;

void main() {
    o_texcoord = texcoord;
    o_normal = normal;
    o_color = color;
    gl_Position = uniforms.transform * vec4(position, 1.0);
}
