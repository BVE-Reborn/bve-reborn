#version 450

layout(early_fragment_tests) in;

#define MAX_NODES 64
#define MAX_SAMPLES 8

layout(location = 0) out vec4 outColor;

struct Node {
    vec4 color;
    float depth;
    int coverage;
    uint next;
};

layout(set = 1, binding = 0, r32ui) uniform uimage2D head_pointers;
layout(set = 1, binding = 1) uniform OIT {
    uint max_nodes;
    uint samples;
};
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
        vec4 sample_color[MAX_SAMPLES];
        bool sample_set[MAX_SAMPLES] = {false, false, false, false, false, false, false, false};
        for (int i = 1; i < count; ++i) {
            for (int s = 0; s < samples; ++s) {
                if ((frags[i].coverage & (1 << s)) != 0) {
                    if (sample_set[s]) {
                        sample_color[s] = mix(sample_color[s], frags[i].color, frags[i].color.a);
                    } else {
                        sample_color[s] = frags[i].color;
                        sample_set[s] = true;
                    }
                }
            }
        }
        vec4 color_sum = vec4(0);
        int sample_count = 0;
        for (; sample_count < samples; ++sample_count) {
            if (sample_set[sample_count]) {
                color_sum += sample_color[sample_count];
            }
        }
        // sample_count should never be zero, as something must have written to some sample
        vec4 color = color_sum / vec4(sample_count);
        outColor = vec4(pow(color.rgb, vec3(1.0 / 2.2)), color.a);
        gl_SampleMask[0] =
            int(sample_set[0]) << 0 |
            int(sample_set[1]) << 1 |
            int(sample_set[2]) << 2 |
            int(sample_set[3]) << 3 |
            int(sample_set[4]) << 4 |
            int(sample_set[5]) << 5 |
            int(sample_set[6]) << 6 |
            int(sample_set[7]) << 7;
    } else {
        discard;
    }
}