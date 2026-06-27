cmake_minimum_required(VERSION 3.20)

project(NightZoom LANGUAGES CXX)

# Windows x64 only.
if(NOT WIN32)
    message(FATAL_ERROR "NightZoom is a Windows x64 ReShade addon and only builds on Windows.")
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(NightZoom SHARED src/main.cpp)

# ReShade addon SDK header + the matching Dear ImGui declarations (v1.92.5).
target_include_directories(NightZoom PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/deps/reshade"
    "${CMAKE_CURRENT_SOURCE_DIR}/deps/imgui")

# winmm        -> timeBeginPeriod/timeEndPeriod
# shell32      -> ShellExecuteA (Discord link)
# windowscodecs + ole32 -> WIC (decode NightZoom_logo.png)
target_link_libraries(NightZoom PRIVATE winmm shell32 windowscodecs ole32)

# Output the addon with ReShade's expected extension: NightZoom.addon64
set_target_properties(NightZoom PROPERTIES
    OUTPUT_NAME "NightZoom"
    SUFFIX ".addon64"
    PREFIX "")
