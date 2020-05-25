#version 450

#define PI_3 1.0471975511965977
#define PI_2 1.5707963267948966
#define PI   3.1415926535626433
#define TAU  6.2831853071795865

layout(location = 0) in vec2 clip_position;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 inv_view_proj;
    float repeats;
};
layout(set = 1, binding = 0) uniform utexture2D skybox;
layout(set = 1, binding = 1) uniform sampler skybox_sampler;

float wrap(float value, float min, float max) {
    return mod(value - min, max - min) + min;
}

void main() {
    // We use the near plane as depth here, as if we used the far plane, it would all NaN out. Doesn't _really_ matter,
    // but 1.0 is a nice round number and results in a depth of 0.1 with my near plane. Good nuf.
    vec4 clip = vec4(clip_position, 1.0, 1.0);
    vec4 world = inv_view_proj * clip;
    world.xyz /= world.w;
    vec3 world_dir = normalize(vec3(world));

    // both [0, tau]
    // normally atan is [-pi, pi] around +z, but we want it [0, tau] around +z, so we need to flip Z and offset it by PI.
    // The normal rotation direction is counter clockwise -pi -> pi, but we need clockwise, so flip the resulting sign to make it pi -> -pi.
    float inv_yaw = -atan(world_dir.x, -world_dir.z) + PI;
    float pitch = asin(world_dir.y) * 2 + PI;

    // Add PI/3 so that the split point is 60 deg to the left of +z
    float x_coord = wrap(inv_yaw + PI_3, 0.0, TAU) / TAU;
    x_coord *= repeats;
    float y_coord;
    if (pitch <= PI) {
        // Below the horizon only gets 25% of the image
        y_coord = (pitch / 2) / TAU;
    }
    else {
        // Above the horizon gets 75% of the image
        y_coord = (pitch / TAU) * 1.5 - 0.5;
    }

    vec4 background_srgb = vec4(texture(usampler2D(skybox, skybox_sampler), vec2(x_coord, 1 - y_coord))) / 255;
    vec3 background = pow(background_srgb.rgb, vec3(2.2));

    outColor = vec4(background, 1.0);
}
