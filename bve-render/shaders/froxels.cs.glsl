#version 450

layout (local_size_x = 4, local_size_y = 4, local_size_z = 4) in;

struct Plane {
    vec3 abc;
    float d;
};

struct Frustum {
    // Left, Right, Top, Bottom, Near, Far
    Plane planes[6];
};

layout(set = 0, binding = 0) uniform Uniforms {
    Frustum frustum;
    uvec4 cluster_size;
};
layout(set = 0, binding = 1) buffer Frustums {
    Frustum result_frustums[];
};

uint get_local_index(uvec3 global, uvec3 total) {
    return global.z * total.x * total.y +
           global.y * total.x +
           global.x;
}

Plane normalize_plane(Plane p) {
    float mag = length(p.abc);

    p.abc /= mag;
    p.d /= mag;

    return p;
}

Plane mix_planes(Plane left, Plane right, float factor) {
    Plane p;
    p.abc = mix(left.abc, right.abc, factor);
    p.d = mix(left.d, right.d, factor);
    return normalize_plane(p);
}

void main() {
    vec3 lerp_start = vec3(gl_GlobalInvocationID) / vec3(cluster_size.xyz);
    vec3 lerp_end = vec3(gl_GlobalInvocationID + 1) / vec3(cluster_size.xyz);

    Plane left = mix_planes(frustum.planes[0], frustum.planes[1], lerp_start.x);
    Plane right = mix_planes(frustum.planes[0], frustum.planes[1], lerp_end.x);

    Plane top = mix_planes(frustum.planes[2], frustum.planes[3], lerp_start.y);
    Plane bottom = mix_planes(frustum.planes[2], frustum.planes[3], lerp_end.y);

    Plane near = mix_planes(frustum.planes[4], frustum.planes[5], lerp_start.z);
    Plane far = mix_planes(frustum.planes[4], frustum.planes[5], lerp_end.z);

    uint index = get_local_index(gl_GlobalInvocationID, cluster_size.xyz);

    Frustum result = Frustum(Plane[6](left, right, top, bottom, near, far));

    result_frustums[index] = result;
}
