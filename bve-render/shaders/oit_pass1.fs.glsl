#version 450

layout(early_fragment_tests) in;

layout(location = 0) in vec2 texcoord;
layout(location = 1) in vec3 normal;
layout(location = 2) flat in vec4 mesh_color;
layout(location = 0) out vec4 outColor;

struct Node {
    vec4 color;
    float depth;
    int coverage;
    uint next;
};

layout(set = 0, binding = 0) uniform utexture2D colorTexture;
layout(set = 0, binding = 1) uniform sampler main_sampler;
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
    vec4 tex_color = pow(vec4(texture(usampler2D(colorTexture, main_sampler), texcoord)) / 255, vec4(2.2));
    vec4 color = tex_color * mesh_color;

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
