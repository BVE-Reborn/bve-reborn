#version 450

layout(location = 0) in vec2 position;

layout(location = 0) out vec2 o_clip_position;

void main() {
    o_clip_position = position;
    gl_Position = vec4(o_clip_position, 1.0, 1.0);
}