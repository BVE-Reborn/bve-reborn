#ifndef GLSL_LIGHTING
#define GLSL_LIGHTING

#include "opaque_signature.glsl"
#include "lights.glsl"
#include "gamma.glsl"

// Flags to enable various lighting features when calling do_lighting()
#define SPECULAR       0x0001
#define SPECULAR_COLOR 0x0010 // Object's color is used in specular
#define DIFFUSE        0x0100
#define AMBIENT        0x1000

vec3 light_with_light(ConeLight light, vec3 object_color, int features) {
    if (bool(light.point)) {
        vec3 light_accumulator = vec3(0.0);

        vec3 light_dir_unnorm = light.location.xyz - view_position.xyz;
        vec3 light_dir = normalize(light_dir_unnorm);
        vec3 normal_ized = normalize(normal);
        if (bool(features & DIFFUSE)) {
            // diffuse
            float diff = max(dot(light_dir, normal_ized), 0.0);
            vec3 diffuse = diff * object_color;
            light_accumulator += diffuse;
        }

        if (bool(features & SPECULAR)) {
            // specular
            vec3 view_dir = normalize(-view_position.xyz);
            vec3 reflect_dir = reflect(-light_dir, normal_ized);
            vec3 halfway_dir = normalize(light_dir + view_dir);
            vec3 specular = vec3(pow(max(dot(normal_ized, halfway_dir), 0.0), 32.0));
            if (bool(features & SPECULAR_COLOR)) {
                specular *= object_color;
            }
            light_accumulator += specular;
        }

        // attenuation
        float distance = length(light_dir_unnorm);
        float attenuation = clamp(1.0 - (distance * distance) / (light.radius * light.radius), 0.0, 1.0);
        attenuation *= attenuation;

        return light.color.rgb * attenuation * light_accumulator;
    } else {
        return vec3(0.0);
    }
}

vec4 do_lighting(int features) {
    uvec3 froxel = compute_froxel();
    uint froxel_index = get_cluster_list_index(froxel, froxel_count);

    vec4 texture_color_srgb = texture(sampler2D(color_texture, main_sampler), texcoord);
    vec4 texture_color = srgb_to_linear(texture_color_srgb);
    vec4 object_color = texture_color * mesh_color;

    vec3 light_accumulation = vec3(0.0);

    uint count = light_index_list[froxel_index].count;
    for (uint l = 0; l < count; ++l) {
        light_accumulation += light_with_light(lights[light_index_list[froxel_index].indices[l]], object_color.rgb, features);
    }

    if (bool(features & AMBIENT)) {
        // ambient
        float factor;
        if (bool(features & SPECULAR_COLOR)) {
            factor = 1.0;
        } else {
            factor = 0.05;
        }
        vec3 ambient = object_color.rgb * factor;
        light_accumulation += ambient;
    }

    return vec4(light_accumulation, object_color.a);
}

#endif