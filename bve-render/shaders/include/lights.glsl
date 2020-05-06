struct PointLight {
    vec3 location;
    float radius;
    float strength;
};

struct ConeLight {
    vec3 location;
    vec3 direction;
    float radius;
    float angle;
    float strength;
};

struct DirecitonalLight {
    vec3 direction;
    float strength;
};
