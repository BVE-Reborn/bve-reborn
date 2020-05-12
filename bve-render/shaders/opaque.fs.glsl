#version 450

#include "opaque_signature.glsl"

void main() {
    vec4 tex_color = pow(vec4(texture(usampler2D(colorTexture, main_sampler), texcoord)) / 255, vec4(2.2));
    vec4 color = tex_color * mesh_color;
    if (color.a <= 0.5) {
        discard;
    } else {
        color.a = 1.0;
    }
    out_color = color;
}
