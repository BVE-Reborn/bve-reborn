#ifndef GLSL_LIGHTS
#define GLSL_LIGHTS

#define MAX_LIGHT_INTEGERS 128
#define MAX_LIGHTS (MAX_LIGHT_INTEGERS - 1)

// Actually a PointLight and ConeLight in one, eliminates a nasty branch
struct ConeLight {
    vec4 location;
    vec4 direction;
    vec4 color;
    float radius;
    float angle;
    int point;
};

struct DirecitonalLight {
    vec4 direction;
    float strength;
};

struct LightIndexSet {
    uint count;
    uint indices[MAX_LIGHTS];
};

#endif
