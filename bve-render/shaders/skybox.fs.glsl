#version 450

layout(location = 0) in vec3 view_position;
layout(location = 0) out vec4 outColor;

layout(set = 1, binding = 1) uniform texture2D skybox;
layout(set = 1, binding = 2) uniform sampler skybox_sampler;

void main() {
    float yaw = acos(dot(vec3(0.0, 0.0, 1.0), normalize(vec3(view_position.x, 0.0, view_position.z))));
    float pitch = acos(dot(vec3(1.0, 0.0, 0.0), normalize(vec3(0.0, view_position.y, view_position.z))));

    outColor = vec4(normalize(view_position), 1.0);
}
