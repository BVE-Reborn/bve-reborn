#version 450

layout (set = 0, binding = 0, rgba8ui) uniform uimage2D mip0;
layout (set = 0, binding = 1, rgba8ui) uniform uimage2D mip1;

void main() {
    ivec2 location = ivec2(gl_GlobalInvocationID.xy);
    ivec2 old_top_left = location * 2;
    vec4 texel00 = vec4(imageLoad(mip0, ivec2(old_top_left.x + 0, old_top_left.y + 0))) / 255;
    vec4 texel10 = vec4(imageLoad(mip0, ivec2(old_top_left.x + 1, old_top_left.y + 0))) / 255;
    vec4 texel01 = vec4(imageLoad(mip0, ivec2(old_top_left.x + 0, old_top_left.y + 1))) / 255;
    vec4 texel11 = vec4(imageLoad(mip0, ivec2(old_top_left.x + 1, old_top_left.y + 1))) / 255;
    texel00.rgb *= ceil(texel00.a);
    texel01.rgb *= ceil(texel01.a);
    texel10.rgb *= ceil(texel10.a);
    texel11.rgb *= ceil(texel11.a);
    vec4 sum = texel00 + texel01 + texel10 + texel11;
    vec4 average = sum / 4;
    imageStore(mip1, location, uvec4(average * 255));
}
