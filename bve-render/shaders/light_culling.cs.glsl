#version 450

#include "frustum.glsl"
#include "lights.glsl"

// x = light
// y = cluster
// work on one cluster at a time, using group sync to add easy sync
layout (local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Uniforms {
    uvec3 cluster_count;
};
layout(set = 0, binding = 1) readonly buffer Frustums {
    Frustum frustums[];
};
layout(set = 0, binding = 2) readonly buffer PointLights {
    PointLight point_lights[];
};
layout(set = 0, binding = 3) readonly buffer ConeLights {
    ConeLight cone_lights[];
};
layout(set = 0, binding = 4) buffer GlobalIndices {
    uint light_index_list[];
};

shared uint group_indices[128];
shared uint group_offset;

void main() {
    if (gl_LocalInvocationID.x == 0) {
        group_offset = 0;
    }

    barrier();

    uvec3 cluster_coords = get_cluser_coords(gl_GlobalInvocationID.y, cluster_count);
}
