cmake_minimum_required(VERSION 3.20)

project(NZ-FPS-Limiter LANGUAGES CXX)

# Windows x64 only.
if(NOT WIN32)
    message(FATAL_ERROR "NightZoom FPS Limiter is a Windows x64 ReShade addon and only builds on Windows.")
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(NZ-FPS-Limiter SHARED src/main.cpp)

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
