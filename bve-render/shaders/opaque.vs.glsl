#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in uvec4 color;
layout(location = 3) in vec2 texcoord;
layout(location = 4) in mat4 model_view_proj;
layout(location = 8) in mat4 model_view;

layout(location = 0) out vec4 o_view_position;
layout(location = 1) out vec4 o_clip_position;
layout(location = 2) out vec2 o_texcoord;
layout(location = 3) out vec3 o_normal;
layout(location = 4) flat out vec4 o_color;

void main() {
    o_texcoord = texcoord;
    o_normal = normal;
    vec4 fcolor = vec4(color) / 255.0;
    o_color = vec4(pow(fcolor.rgb, vec3(2.2)), fcolor.a);
    o_view_position = model_view * vec4(position, 1.0);
    vec4 pre_clip = model_view_proj * vec4(position, 1.0);
    gl_Position = pre_clip;
    o_clip_position = pre_clip;
}
