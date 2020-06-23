#ifndef GLSL_GAMMA
#define GLSL_GAMMA

vec4 rgbaU8_to_rgbaF32(uvec4 u8) {
    return vec4(u8) / 255;
}

uvec4 rgbaF32_to_rgbaU8(vec4 f32) {
    return uvec4(round(f32 * 255));
}

vec4 srgb_to_linear(vec4 srgb) {
    vec3 color_srgb = srgb.rgb;
    vec3 selector = ceil(color_srgb - 0.04045); // 0 if under value, 1 if over
    vec3 under = color_srgb / 12.92;
    vec3 over = pow((color_srgb + 0.055) / 1.055, vec3(2.4));
    vec3 result = mix(under, over, selector);
    return vec4(pow(srgb.rgb, vec3(2.2)), srgb.a);
}

vec4 linear_to_srgb(vec4 linear) {
    vec3 color_linear = linear.rgb;
    vec3 selector = ceil(color_linear - 0.0031308); // 0 if under value, 1 if over
    vec3 under = 12.92 * color_linear;
    vec3 over = 1.055 * pow(color_linear, vec3(0.41666)) - 0.055;
    vec3 result = mix(under, over, selector);
    return vec4(result, linear.a);
}

#endif