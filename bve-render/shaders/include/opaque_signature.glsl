#ifndef GLSL_OPAQUE_SIGNATURE
#define GLSL_OPAQUE_SIGNATURE

#include "frustum.glsl"
#include "lights.glsl"

layout(location = 0) in vec4 view_position;
layout(location = 1) in vec4 clip_position;
layout(location = 2) in vec2 texcoord;
layout(location = 3) in vec3 normal;
layout(location = 4) flat in vec4 mesh_color;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform utexture2D color_texture;
layout(set = 0, binding = 1) uniform sampler main_sampler;
layout(set = 1, binding = 0) uniform FroxelUniforms {
    uvec3 froxel_count;
    float max_depth;
};
layout(set = 1, binding = 1) readonly buffer Frustums {
    Frustum frustums[];
};
layout(set = 1, binding = 2) readonly buffer Lights {
    ConeLight lights[];
};
layout(set = 1, binding = 3) readonly buffer LightList {
    LightIndexSet light_index_list[];
};

vec3 get_clip_position() {
    return clip_position.xyz / clip_position.w;
}

uvec3 compute_froxel() {
    // clip position but [0, 1] in xy
    vec2 scale = get_clip_position().xy * 0.5 + 0.5;
    vec2 frustum_raw = scale * vec2(froxel_count.xy);
    uvec2 frustum_xy = uvec2(floor(frustum_raw));
    // length(view - camera)
    float depth = length(view_position.xyz);
    uint depth_frustum = uint(floor((depth / max_depth) * froxel_count.z));
    return uvec3(frustum_xy, depth_frustum);
}

#endif
