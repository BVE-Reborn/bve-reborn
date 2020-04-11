#version 450

layout(location = 0) in vec2 texcoord;
layout(location = 1) in vec3 normal;
layout(location = 2) flat in vec4 mesh_color_srgb;
layout(location = 3) flat in vec2 texcoord_boundries_min;
layout(location = 4) flat in vec2 texcoord_boundries_max;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 transform;
    int transparent;
} uniforms;
layout(set = 0, binding = 1) uniform utexture2D colorTexture;
layout(set = 0, binding = 2) uniform sampler main_sampler;

vec2 wrap(vec2 value, vec2 min, vec2 max) {
    return mod(value - min, max - min) + min;
}

void main() {
    vec2 wrapped_texcoord = wrap(texcoord, texcoord_boundries_min, texcoord_boundries_max);
    vec4 tex_color = pow(vec4(texture(usampler2D(colorTexture, main_sampler), wrapped_texcoord)) / 255, vec4(2.2));
    vec4 mesh_color = pow(mesh_color_srgb, vec4(2.2));
    vec4 color = tex_color * mesh_color;
    if (!bool(uniforms.transparent) && color.a <= 0.5) {
        discard;
    }
    outColor = color;
}
