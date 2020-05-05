#include "frustum.glsl"

layout(location = 0) in vec2 texcoord;
layout(location = 1) in vec3 normal;
layout(location = 2) flat in vec4 mesh_color;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform utexture2D colorTexture;
layout(set = 0, binding = 1) uniform sampler main_sampler;
layout(set = 1, binding = 0) readonly buffer Frustums {
    Frustum result_frustums[];
};
layout(set = 1, binding = 1) buffer FrustumUniforms {
    uvec4 frustum_count;
};
