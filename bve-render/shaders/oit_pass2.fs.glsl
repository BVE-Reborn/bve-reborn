#version 450

#include "gamma.glsl"

layout(early_fragment_tests) in;

layout(location = 0) out vec4 outColor;

struct Node {
    vec4 color;
    float depth;
    int coverage;
    uint next;
};

layout(set = 0, binding = 0, r32ui) uniform uimage2D head_pointers;
layout(set = 0, binding = 1) uniform OIT {
    uint max_nodes;
    uint samples;
};
layout(set = 0, binding = 2, std430) buffer NodeBuffer {
    uint next_index;
    Node nodes[];
};
#if MAX_SAMPLES != 1
    layout(set = 1, binding = 0) uniform texture2DMS framebuffer;
#else
    layout(set = 1, binding = 0) uniform texture2D framebuffer;
#endif
layout(set = 1, binding = 1) uniform sampler framebuffer_sampler;

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
        while(j > 0 && to_insert.depth > frags[j-1].depth) {
            frags[j] = frags[j-1];
            j--;
        }
        frags[j] = to_insert;
    }

    vec4 sample_color[MAX_SAMPLES];
    for (int s = 0; s < MAX_SAMPLES; ++s) {
        #if MAX_SAMPLES != 1
            sample_color[s] = texelFetch(sampler2DMS(framebuffer, framebuffer_sampler), ivec2(gl_FragCoord.xy), s);
        #else
            sample_color[s] = texelFetch(sampler2D(framebuffer, framebuffer_sampler), ivec2(gl_FragCoord.xy), s);
        #endif
    }
    for (int i = 0; i < count; ++i) {
        for (int s = 0; s < MAX_SAMPLES; ++s) {
            if ((frags[i].coverage & (1 << s)) != 0) {
                sample_color[s] = mix(sample_color[s], frags[i].color, frags[i].color.a);
            }
        }
    }
    vec4 color_sum = vec4(0);
    for (int s = 0; s < MAX_SAMPLES; ++s) {
        color_sum += sample_color[s];
    }
    vec4 color = color_sum / vec4(MAX_SAMPLES);
    outColor = linear_to_srgb(color);
}
