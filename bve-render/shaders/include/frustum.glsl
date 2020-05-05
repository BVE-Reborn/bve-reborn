struct Plane {
    vec3 abc;
    float d;
};

struct Frustum {
    // Left, Right, Top, Bottom
    Plane planes[4];
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
