#version 450

#include "opaque_signature.glsl"

uvec2 find_real_frustum() {
    for (int x = 0; x < froxel_count.x; ++x) {
        for (int y = 0; y < froxel_count.y; ++y) {
            uint frustum_index = get_frustum_list_index(uvec2(x, y), froxel_count.xy);
            Frustum frustum = frustums[frustum_index];
            if (contains_point(frustum, vec3(view_position))) {
                return uvec2(x, y);
            }
        }
    }
    return uvec2(1000, 1000);
}

void main() {
    uvec2 real = find_real_frustum();
    uvec2 computed = compute_froxel().xy;
    if (real == computed) {
        out_color = vec4(0.0, 1.0, 0.0, 1.0);
    } else {
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
}
