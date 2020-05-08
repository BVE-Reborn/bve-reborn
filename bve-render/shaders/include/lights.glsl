#define MAX_LIGHT_INTEGERS 128
#define MAX_LIGHTS (MAX_LIGHT_INTEGERS - 1)

// Actually a PointLight and ConeLight in one, eliminates a nasty branch
struct ConeLight {
    vec3 location;
    vec3 direction;
    float radius;
    float angle;
    float strength;
    bool point;
};

struct DirecitonalLight {
    vec3 direction;
    float strength;
};

struct LightIndexSet {
    uint count;
    uint indices[MAX_LIGHTS];
};