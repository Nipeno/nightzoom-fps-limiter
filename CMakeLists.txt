cmake_minimum_required(VERSION 3.20)

# Addon version. Single source of truth: the git tag vX.Y.Z, which CI passes as
# -DNZ_VERSION=X.Y.Z. Untagged/local builds fall back to 0.0.0. This flows into the
# DLL's VERSIONINFO resource (so the installer can read what's installed) and into the
# overlay's "vX.Y.Z" label via NZ_VERSION_STR.
set(NZ_VERSION "0.0.0" CACHE STRING "NightZoom FPS Limiter version (X.Y.Z)")

project(NZ-FPS-Limiter VERSION ${NZ_VERSION} LANGUAGES CXX RC)

# Windows x64 only.
if(NOT WIN32)
    message(FATAL_ERROR "NightZoom FPS Limiter is a Windows x64 ReShade addon and only builds on Windows.")
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Comma-separated form (a.b.c.0) for the binary FILEVERSION/PRODUCTVERSION fields of the
# VERSIONINFO resource, generated from version.rc.in.
set(NZ_VERSION_COMMA "${PROJECT_VERSION_MAJOR},${PROJECT_VERSION_MINOR},${PROJECT_VERSION_PATCH},0")
configure_file("${CMAKE_CURRENT_SOURCE_DIR}/src/version.rc.in" "${CMAKE_CURRENT_BINARY_DIR}/version.rc" @ONLY)

add_library(NZ-FPS-Limiter SHARED src/main.cpp "${CMAKE_CURRENT_BINARY_DIR}/version.rc")

# Expose the version string to the C++ source (overlay label).
target_compile_definitions(NZ-FPS-Limiter PRIVATE NZ_VERSION_STR="${PROJECT_VERSION}")

# ReShade addon SDK header + the matching Dear ImGui declarations (v1.92.5).
target_include_directories(NZ-FPS-Limiter PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/deps/reshade"
    "${CMAKE_CURRENT_SOURCE_DIR}/deps/imgui")

# winmm        -> timeBeginPeriod/timeEndPeriod
# shell32      -> ShellExecuteA (Discord link)
# windowscodecs + ole32 -> WIC (decode NZ-FPS-Limiter_logo.png)
target_link_libraries(NZ-FPS-Limiter PRIVATE winmm shell32 windowscodecs ole32)

# Output the addon with ReShade's expected extension: NZ-FPS-Limiter.addon64
set_target_properties(NZ-FPS-Limiter PROPERTIES
    OUTPUT_NAME "NZ-FPS-Limiter"
    SUFFIX ".addon64"
    PREFIX "")
