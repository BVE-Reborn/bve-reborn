#version 450

#include "opaque_signature.glsl"

void main() {
    uvec3 froxel = compute_froxel();
    uint count = light_index_list[get_cluster_list_index(froxel, froxel_count)].count;
    vec4 scaled = vec4(float(count) / 10.0, 0.0, 0.0, 1.0);

    out_color = pow(scaled, vec4(2.2));
}
