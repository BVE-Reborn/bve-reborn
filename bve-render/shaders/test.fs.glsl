#version 450

layout(location = 0) in vec2 texcoord;
layout(location = 1) in vec3 normal;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform texture2D color;
layout(set = 0, binding = 2) uniform sampler main_sampler;

void main() {
    vec4 tex = texture(sampler2D(color, main_sampler), texcoord);
    if (tex.a == 0.0) {
        discard;
    }
    outColor = tex;
}
