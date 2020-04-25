#version 450

layout(early_fragment_tests) in;

#define MAX_NODES 8
#define MAX_SAMPLES 8

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
layout(set = 1, binding = 0) uniform texture2DMS framebuffer;
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
        // This is inverted as I used an inverted depth buffer
        while(j > 0 && to_insert.depth < frags[j-1].depth) {
            frags[j] = frags[j-1];
            j--;
        }
        frags[j] = to_insert;
    }


    vec4 sample_color[MAX_SAMPLES];
    for (int s = 0; s < samples; ++s) {
        sample_color[s] = texelFetch(sampler2DMS(framebuffer, framebuffer_sampler), ivec2(gl_FragCoord.xy), s);
    }
    for (int i = 0; i < count; ++i) {
        for (int s = 0; s < samples; ++s) {
            if ((frags[i].coverage & (1 << s)) != 0) {
                sample_color[s] = mix(sample_color[s], frags[i].color, frags[i].color.a);
            }
        }
    }
    vec4 color_sum = vec4(0);
    for (int s = 0; s < samples; ++s) {
        color_sum += sample_color[s];
    }
    vec4 color = color_sum / vec4(samples);
    outColor = vec4(pow(color.rgb, vec3(1.0 / 2.2)), color.a);
}
