#version 450

#include "gamma.glsl"

layout (local_size_x = 8, local_size_y = 8) in;

layout (set = 0, binding = 0, rgba8ui) uniform uimage2D mip0;
layout (set = 0, binding = 1, rgba8ui) uniform uimage2D mip1;
layout (set = 0, binding = 2) uniform Locals {
    uvec2 texture_dimensions;
};

vec4 load_gamma(ivec2 position) {
    return srgb_to_linear(rgbaU8_to_rgbaF32(imageLoad(mip0, position)));
}

void store_gamma(ivec2 location, vec4 value) {
    imageStore(mip1, location, rgbaF32_to_rgbaU8(linear_to_srgb(value)));
}

void main() {
    ivec2 location = ivec2(gl_GlobalInvocationID.xy);
    if (!(location.x < texture_dimensions.x && location.y < texture_dimensions.y)) {
        return;
    }
    ivec2 old_top_left = location * 2;
    vec4 texel00 = load_gamma(ivec2(old_top_left.x + 0, old_top_left.y + 0));
    vec4 texel10 = load_gamma(ivec2(old_top_left.x + 1, old_top_left.y + 0));
    vec4 texel01 = load_gamma(ivec2(old_top_left.x + 0, old_top_left.y + 1));
    vec4 texel11 = load_gamma(ivec2(old_top_left.x + 1, old_top_left.y + 1));
    vec4 sum = texel00 + texel01 + texel10 + texel11;
    vec4 average = sum / 4;
    store_gamma(location, average);
}
