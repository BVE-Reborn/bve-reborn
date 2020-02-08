cmake_minimum_required(VERSION 3.12)

add_subdirectory(sdl2)

project(bve-rex-sys)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

find_package(SDL2 REQUIRED SDL_BUILDING_LIBRARY)

file(GLOB_RECURSE SOURCES LIST_DIRECTORIES false CONFIGURE_DEPENDS rex/src/*.cpp rex/src/*.h rex/src/*.c wrapper/*)

message(STATUS "${SOURCES}")

add_library(bverex SHARED ${SOURCES})
target_include_directories(bverex PUBLIC rex/src ${CMAKE_SOURCE_DIR}/sdl2/include)
target_link_libraries(bverex PUBLIC SDL2)

if(NOT WIN32)
    target_compile_definitions(bverex PUBLIC -D_DEFAULT_SOURCE)
endif()

install(TARGETS bverex DESTINATION ${CMAKE_INSTALL_PREFIX})
