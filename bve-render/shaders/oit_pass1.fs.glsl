#version 450

layout(early_fragment_tests) in;

#include "opaque_signature.glsl"
#include "do_lighting.glsl"

struct Node {
    vec4 color;
    float depth;
    int coverage;
    uint next;
};

layout(set = 2, binding = 0, r32ui) uniform uimage2D head_pointers;
layout(set = 2, binding = 1) uniform OIT {
    uint max_nodes;
    uint samples;
};
layout(set = 2, binding = 2, std430) buffer NodeBuffer {
    uint next_index;
    Node nodes[];
};

void main() {
    vec4 color = do_lighting();

    if (color.a <= 0.0) {
        discard;
    }

    uint node_idx = atomicAdd(next_index, 1);
    if (node_idx < max_nodes) {
        uint prev_head = imageAtomicExchange(head_pointers, ivec2(gl_FragCoord.xy), node_idx);

        nodes[node_idx].color = color;
        nodes[node_idx].depth = gl_FragCoord.z;
        nodes[node_idx].coverage = gl_SampleMaskIn[0];
        nodes[node_idx].next = prev_head;
    }
}
