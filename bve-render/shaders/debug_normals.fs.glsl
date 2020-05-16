#version 450

#include "opaque_signature.glsl"

void main() {
    out_color = vec4(pow(max(normal, vec3(0.0)), vec3(2.2)), 1.0);
}
