struct Plane {
    vec3 abc;
    float d;
};

Plane normalize_plane(Plane p) {
    float mag = length(p.abc);

    p.abc /= mag;
    p.d /= mag;

    return p;
}

Plane compute_plane(vec3 p0, vec3 p1, vec3 p2) {
    vec3 v0 = p1 - p0;
    vec3 v1 = p2 - p0;

    vec3 normal = normalize(cross(v0, v1));

    // Apply the plane equation to one of the points to get the offset
    float d = dot(normal, p0);

    return Plane(normal, d);
}

float distance(Plane plane, vec3 point) {
    return dot(plane.abc, point) + plane.d;
}

struct Frustum {
    // Left, Right, Top, Bottom
    Plane planes[4];
};

bool contains_point(Frustum frustum, vec3 point) {
    bool res = true;
    for (int i = 0; i < 4; ++i) {
        if (distance(frustum.planes[i], point) <= 0) {
            res = false;
        }
    }
    return res;
}
