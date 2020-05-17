#version 450

#include "opaque_signature.glsl"
#include "gamma.glsl"

void main() {
    out_color = srgb_to_linear(max(vec4(normal, 1.0), vec4(0.0)));
}
