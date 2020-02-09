cmake_minimum_required(VERSION 3.12)

project(bve-rex-sys)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

set(CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS true)
set(CMAKE_INSTALL_RPATH_USE_LINK_PATH true)

file(GLOB_RECURSE SOURCES LIST_DIRECTORIES false CONFIGURE_DEPENDS rex/src/*.cpp rex/src/*.h rex/src/*.c wrapper/*)

message(STATUS "${SOURCES}")

add_library(bverex SHARED ${SOURCES})
target_include_directories(bverex PUBLIC rex/src "${SDL2_INCLUDE_DIR}")
target_link_directories(bverex PUBLIC "${SDL2_LINK_SEARCH}")
target_link_libraries(bverex PUBLIC SDL2)

if(WIN32)
    target_compile_definitions(bverex PUBLIC -D_CRT_SECURE_NO_WARNINGS)
else()
    target_compile_definitions(bverex PUBLIC -D_DEFAULT_SOURCE)
endif()

install(TARGETS bverex DESTINATION ${CMAKE_INSTALL_PREFIX})
