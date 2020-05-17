#ifndef GLSL_GAMMA
#define GLSL_GAMMA

vec4 rgbaU8_to_rgbaF32(uvec4 u8) {
    return vec4(u8) / 255;
}

uvec4 rgbaF32_to_rgbaU8(vec4 f32) {
    return uvec4(round(f32 * 255));
}

vec4 srgb_to_linear(vec4 srgb) {
    return vec4(pow(srgb.rgb, vec3(2.2)), srgb.a);
}

vec4 linear_to_srgb(vec4 linear) {
    return vec4(pow(linear.rgb, vec3(1 / 2.2)), linear.a);
}

#endif