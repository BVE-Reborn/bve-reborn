#version 450

#include "opaque_signature.glsl"
#include "do_lighting.glsl"
#include "gamma.glsl"

layout(early_fragment_tests) in;

struct Node {
    vec4 color;
    float depth;
    int coverage;
    uint next;
};

layout(set = 2, binding = 0) buffer HeadPointers {
    uint head_pointers[];
};
layout(set = 2, binding = 1) uniform OIT {
    uint max_nodes;
    uint samples;
    uvec2 screen_size;
};
layout(set = 2, binding = 2, std430) buffer NodeBuffer {
    uint next_index;
    Node nodes[];
};

void main() {
    vec4 color = do_lighting(AMBIENT | SPECULAR | SPECULAR_COLOR | DIFFUSE);

    if (color.a <= 0.0) {
        discard;
    }

    uint node_idx = atomicAdd(next_index, 1);
    if (node_idx < max_nodes) {
        uint prev_head = atomicExchange(head_pointers[uint(gl_FragCoord.x) * screen_size.y + uint(gl_FragCoord.y)], node_idx);

        nodes[node_idx].color = color;
        nodes[node_idx].depth = gl_FragCoord.z;
        nodes[node_idx].coverage = gl_SampleMaskIn[0];
        nodes[node_idx].next = prev_head;
    }
}
