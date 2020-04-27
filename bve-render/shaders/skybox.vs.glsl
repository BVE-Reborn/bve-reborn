#version 450

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 o_view_position;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 mvp;
    mat4 mv;
};

void main() {
    o_view_position = mat3(mv) * position;
    vec4 clip = mvp * vec4(position, 1.0);
    // Perform perspective divide myself so I can set z to 0.0
    gl_Position = clip;
}
