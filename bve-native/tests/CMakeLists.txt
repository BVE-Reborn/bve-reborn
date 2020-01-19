cmake_minimum_required(VERSION 3.2)

project(bve-native-test LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 11)

include_directories(${CMAKE_SOURCE_DIR}/../include)
link_directories(${CMAKE_SOURCE_DIR}/../../target/debug/)

add_executable(bve-native-c main.c)
add_executable(bve-native-cpp main.cpp)

if (WIN32)
    target_link_libraries(bve-native-c ${CMAKE_SOURCE_DIR}/../../target/debug/bve_native.dll)
    target_link_libraries(bve-native-cpp ${CMAKE_SOURCE_DIR}/../../target/debug/bve_native.dll)
    file(COPY "${CMAKE_SOURCE_DIR}/../../target/debug/bve_native.dll" DESTINATION ${CMAKE_BINARY_DIR})
elseif(APPLE)
    target_link_libraries(bve-native-c ${CMAKE_SOURCE_DIR}/../../target/debug/libbve_native.dylib)
    target_link_libraries(bve-native-cpp ${CMAKE_SOURCE_DIR}/../../target/debug/libbve_native.dylib)
elseif(UNIX)
    target_link_libraries(bve-native-c ${CMAKE_SOURCE_DIR}/../../target/debug/libbve_native.so)
    target_link_libraries(bve-native-cpp ${CMAKE_SOURCE_DIR}/../../target/debug/libbve_native.so)
else()
    message(FATAL_ERROR "Unsupported platform")
endif()