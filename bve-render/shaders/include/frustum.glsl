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

    vec3 normal = -normalize(cross(v0, v1));

    // Apply the plane equation to one of the points to get the offset
    float d = dot(normal, p0);

    return Plane(normal, d);
}

float distance(Plane plane, vec3 point) {
    return dot(plane.abc, point) + plane.d;
}

struct Sphere {
    vec3 location;
    float radius;
};

struct Frustum {
    // Left, Right, Top, Bottom
    Plane planes[4];
};

bool contains_point(Frustum frustum, vec3 point) {
    bool res = true;
    for (int i = 0; i < 4; ++i) {
        if (distance(frustum.planes[i], point) < 0) {
            res = false;
        }
    }
    return res;
}

bool contains_sphere(Frustum frustum, Sphere sphere) {
    bool res = true;
    for (int i = 0; i < 4; ++i) {
        if (distance(frustum.planes[i], sphere.location) < -sphere.radius) {
            res = false;
        }
    }
    return res;
}

uint get_frustum_list_index(uvec2 coords, uvec2 total) {
    return coords.y * total.x +
           coords.x;
}

uvec2 get_frustum_coords(uint index, uvec2 total) {
    uint y = (index / total.x) % total.y;
    uint x = index % total.x;
    return uvec2(x, y);
}

uint get_cluster_list_index(uvec3 coords, uvec3 total) {
    return coords.z * total.x * total.y +
           coords.y * total.x +
           coords.x;
}

uvec3 get_cluster_coords(uint index, uvec3 total) {
    uint z = index / (total.x * total.y);
    uint y = (index / total.x) % total.y;
    uint x = index % total.x;
    return uvec3(x, y, z);
}

struct ZBounds {
    float start;
    float end;
};

ZBounds get_zbounds(uint z_number, uint z_divisions, float max_depth) {
    float start = float(z_number) * (max_depth / float(z_divisions));
    float end = float(z_number + 1) * (max_depth / float(z_divisions));
    return ZBounds(start, end);
}

bool contains_sphere(ZBounds bounds, Sphere sphere) {
    float depth = length(sphere.location);
    if (depth - sphere.radius > bounds.end) {
        return false;
    } else if (depth + sphere.radius < bounds.start) {
        return false;
    } else {
        return true;
    }
}

