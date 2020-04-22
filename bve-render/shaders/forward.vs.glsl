#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in uvec4 color;
layout(location = 3) in vec2 texcoord;
layout(location = 4) in mat4 transform;
layout(location = 8) in int transparent;

layout(location = 0) out vec2 o_texcoord;
layout(location = 1) out vec3 o_normal;
layout(location = 2) flat out vec4 o_color;
layout(location = 3) flat out int o_transparent;

void main() {
    o_texcoord = texcoord;
    o_normal = normal;
    vec4 fcolor = vec4(color) / 255.0;
    o_color = vec4(pow(fcolor.rgb, vec3(2.2)), fcolor.a);
    o_transparent = transparent;
    gl_Position = transform * vec4(position, 1.0);
}
