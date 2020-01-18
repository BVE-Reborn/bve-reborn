cmake_minimum_required(VERSION 3.12)

project(bve-native-test LANGUAGES C CXX)

include_directories(${CMAKE_SOURCE_DIR}/../include)
link_directories(${CMAKE_SOURCE_DIR}/../../target/debug/)
link_libraries(bve_native)

add_executable(bve-native-c main.c)
add_executable(bve-native-cpp main.cpp)