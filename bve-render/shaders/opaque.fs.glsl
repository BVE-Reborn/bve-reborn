#version 450

layout(location = 0) in vec2 texcoord;
layout(location = 1) in vec3 normal;
layout(location = 2) flat in vec4 mesh_color;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform utexture2D colorTexture;
layout(set = 0, binding = 1) uniform sampler main_sampler;

void main() {
    vec4 tex_color = pow(vec4(texture(usampler2D(colorTexture, main_sampler), texcoord)) / 255, vec4(2.2));
    vec4 color = tex_color * mesh_color;
    if (color.a <= 0.5) {
        discard;
    } else {
        color.a = 1.0;
    }
    outColor = vec4(pow(color.rgb, vec3(1 / 2.2)), color.a);
}
