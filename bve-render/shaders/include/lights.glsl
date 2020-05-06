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
