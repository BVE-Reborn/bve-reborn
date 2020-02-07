# This cmake isn't for building. Don't build with it, it's for CLion to understand wtf is going on

cmake_minimum_required(VERSION 3.12)

project(bve-rex-sys)

file(GLOB_RECURSE SOURCES LIST_DIRECTORIES false CONFIGURE_DEPENDS rex/src/*.cpp rex/src/*.h rex/src*.c wrapper/*)

add_library(bve-rex-sys ${SOURCES})
target_include_directories(bve-rex-sys PUBLIC rex/src)
