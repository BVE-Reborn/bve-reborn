#version 450

#include "frustum.glsl"
#include "lights.glsl"

#define THREADS 64

// Work on one light at a time, across 64 clusters
// x = light
// y = cluster
layout (local_size_x = 1, local_size_y = THREADS, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Uniforms {
    uvec3 cluster_count;
    uint total_lights;
    float max_depth;
};
layout(set = 0, binding = 1) readonly buffer Frustums {
    Frustum frustums[];
};
layout(set = 0, binding = 2) readonly buffer Lights {
    ConeLight lights[];
};
layout(set = 0, binding = 3) buffer GlobalIndices {
    LightIndexSet light_index_list[];
};


void main() {
    uint cluster_index = gl_GlobalInvocationID.y;
    uvec3 cluster_coords = get_cluster_coords(cluster_index, cluster_count);
    uint frustum_index = get_frustum_list_index(cluster_coords.xy, cluster_count.xy);

    ZBounds z_bounds = get_zbounds(cluster_coords.z, cluster_count.z, max_depth);
    Frustum frustum = frustums[frustum_index];

    uint light_count = 0;
    for (uint i = 0; i < total_lights; i++) {
        ConeLight light = lights[i];
        Sphere sphere = Sphere(light.location.xyz, light.radius);

        if (contains_sphere(frustum, sphere) && contains_sphere(z_bounds, sphere)) {
            if (light_count < MAX_LIGHTS) {
                light_index_list[cluster_index].indices[light_count] = i;
                light_count += 1;
            } else {
                break;
            }
        }
    }

    light_index_list[cluster_index].count = light_count;
}
