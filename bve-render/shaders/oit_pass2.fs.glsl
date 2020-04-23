#version 450

layout(early_fragment_tests) in;

#define MAX_NODES 64

layout(location = 0) out vec4 outColor;

struct Node {
    vec4 color;
    float depth;
    uint next;
};

layout(set = 1, binding = 0, r32ui) uniform uimage2D head_pointers;
layout(set = 1, binding = 2, std430) buffer NodeBuffer {
    uint next_index;
    Node nodes[];
};

void main() {
    Node frags[MAX_NODES];

    uint n = imageLoad(head_pointers, ivec2(gl_FragCoord.xy)).r;
    imageStore(head_pointers, ivec2(gl_FragCoord.xy), uvec4(0xFFFFFFFF));

    int count = 0;
    for(; n != 0xFFFFFFFF && count < MAX_NODES; ++count) {
        frags[count] = nodes[n];
        n = frags[count].next;
    }

    // insertion sort
    for(uint i = 1; i < count; i++) {
        Node to_insert = frags[i];
        uint j = i;
        // This is inverted as I used an inverted depth buffer
        while(j > 0 && to_insert.depth < frags[j-1].depth) {
            frags[j] = frags[j-1];
            j--;
        }
        frags[j] = to_insert;
    }

    if (count != 0) {
        vec4 color = frags[0].color;
        for (int i = 1; i < count; ++i) {
            color = mix(color, frags[i].color, frags[i].color.a);
        }
        outColor = vec4(pow(color.rgb, vec3(1.0 / 2.2)), color.a);
    } else {
        discard;
    }
}
