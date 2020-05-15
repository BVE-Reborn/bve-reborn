#pragma once

#if defined(__cplusplus) && (__cplusplus >= 201703)
	#define BVE_NO_DISCARD [[nodiscard]]
#elif defined(__GNUC__)
	#define BVE_NO_DISCARD __attribute__ ((warn_unused_result))
#elif defined(_MSC_VER)
	#define BVE_NO_DISCARD _Check_return_
#else
	#define BVE_NO_DISCARD
#endif

typedef struct {
    float x;
    float y;
} BVE_Vec2;

typedef struct {
    float x;
    float y;
    float z;
    float _z;
} BVE_Vec3;

typedef struct {
    float x;
    float y;
    float z;
    float w;
} BVE_Vec4;
