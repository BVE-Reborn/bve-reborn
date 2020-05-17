vec3 light_with_light(ConeLight light, vec3 object_color) {
    if (bool(light.point)) {
        // diffuse
        vec3 light_dir = normalize(light.location.xyz - view_position.xyz);
        vec3 normal_ized = normalize(normal);
        float diff = max(dot(light_dir, normal_ized), 0.0);
        vec3 diffuse = diff * object_color;

        // specular
        vec3 view_dir = normalize(-view_position.xyz);
        vec3 reflect_dir = reflect(-light_dir, normal_ized);
        vec3 halfway_dir = normalize(light_dir + view_dir);
        vec3 specular = pow(max(dot(normal_ized, halfway_dir), 0.0), 32.0) * vec3(0.3); // fix constant

        return specular + diffuse;
    } else {
        return vec3(0.0);
    }
}

vec4 do_lighting() {
    uvec3 froxel = compute_froxel();
    uint froxel_index = get_cluster_list_index(froxel, froxel_count);

    vec4 texture_color_srgb = texture(usampler2D(color_texture, main_sampler), texcoord);
    vec4 texture_color = vec4(pow(texture_color_srgb.rgb / 255, vec3(2.2)), texture_color_srgb.a);
    vec3 object_color = texture_color.rgb * mesh_color.rgb;

    vec3 light_accumulation = vec3(0.0);

    uint count = light_index_list[froxel_index].count;
    for (uint l = 0; l < count; ++l) {
        light_accumulation += light_with_light(lights[light_index_list[froxel_index].indices[l]], object_color);
    }

    // ambient
    vec3 ambient = object_color * 0.05;
    light_accumulation += ambient;

    return vec4(light_accumulation, texture_color.a);
}