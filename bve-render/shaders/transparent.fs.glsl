#version 450

#include "opaque_signature.glsl"
#include "do_lighting.glsl"

void main() {
    vec4 color = do_lighting(SPECULAR | DIFFUSE | AMBIENT);
    out_color = color;
}
