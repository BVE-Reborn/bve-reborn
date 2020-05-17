#version 450

#include "opaque_signature.glsl"
#include "do_lighting.glsl"

void main() {
    vec4 color = do_lighting();
    if (color.a <= 0.5) {
        discard;
    } else {
        color.a = 1.0;
    }
    out_color = color;
}
