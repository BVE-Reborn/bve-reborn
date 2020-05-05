struct Plane {
    vec3 abc;
    float d;
};

struct Frustum {
    // Left, Right, Top, Bottom
    Plane planes[4];
};
