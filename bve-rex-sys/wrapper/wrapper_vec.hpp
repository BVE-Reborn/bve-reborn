#ifndef BVE_NATIVE_TEST_WRAPPER_VEC_H
#define BVE_NATIVE_TEST_WRAPPER_VEC_H

/// <div rustbindgen replaces="rx::math::vec2"></div>
template<typename T>
struct vec2 {
    T x;
    T y;
};

/// <div rustbindgen replaces="rx::math::vec3"></div>
template<typename T>
struct vec3 {
    T x;
    T y;
    T z;
};

/// <div rustbindgen replaces="rx::math::vec4"></div>
template<typename T>
struct vec4 {
    T x;
    T y;
    T z;
    T w;
};

#endif //BVE_NATIVE_TEST_WRAPPER_VEC_H
