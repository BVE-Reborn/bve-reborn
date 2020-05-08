#version 450

#include "frustum.glsl"
#include "lights.glsl"

#define THREADS 64

// Work on one cluster at a time, building the index array in group shared memory.
// We never issue more than one workgroup wide in x. One workgroup = one cluster
// x = light % 64
// y = cluster
layout (local_size_x = THREADS, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Uniforms {
    uvec3 cluster_count;
    uint light_count;
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

shared uint group_indices[MAX_LIGHTS];
shared uint group_offset;

void main() {
    if (gl_LocalInvocationID.x == 0) {
        group_offset = 0;
    }

    barrier();

    uint local_index = gl_LocalInvocationID.x;
    uint cluster_index = gl_GlobalInvocationID.y;
    uvec3 cluster_coords = get_cluster_coords(cluster_index, cluster_count);
    uint frustum_index = get_frustum_list_index(cluster_coords.xy, cluster_count.xy);

    ZBounds z_bounds = get_zbounds(cluster_coords.z, cluster_count.z, max_depth);
    Frustum frustum = frustums[frustum_index];

    for (uint light_index = local_index; light_index < light_count; light_index += THREADS) {
        ConeLight light = lights[light_index];
        Sphere sphere = Sphere(light.location, light.radius);

        if (contains_sphere(frustum, sphere) && contains_sphere(z_bounds, sphere)) {
            uint index = atomicAdd(group_offset, 1);
            if (index < 128) {
                group_indices[index] = light_index;
                break;
            }
        }
    }

    barrier();

    if (local_index == 0) {
        light_index_list[cluster_index].count = group_offset;
    }

    for (uint index = local_index; index < group_offset; index += THREADS) {
        light_index_list[cluster_index].indices[local_index] = group_indices[index];
    }
}
