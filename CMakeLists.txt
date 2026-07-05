cmake_minimum_required(VERSION 3.25)

project(VidCut VERSION 0.1.0 LANGUAGES CXX RC)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

# Compiler flags
if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(-Wall -Wextra -Wpedantic)
elseif(MSVC)
    add_compile_options(/W4 /utf-8)
    add_compile_definitions(_CRT_SECURE_NO_WARNINGS NOMINMAX)
endif()

# Qt6
find_package(Qt6 REQUIRED COMPONENTS Core Widgets Multimedia MultimediaWidgets)

# FFmpeg — optional during UI-focus phase
option(VIDCUT_ENABLE_FFMPEG "Build with FFmpeg media support" OFF)

set(FFMPEG_TARGETS "")
if(VIDCUT_ENABLE_FFMPEG)
    find_package(PkgConfig QUIET)
    if(PkgConfig_FOUND)
        pkg_check_modules(AVCODEC    REQUIRED IMPORTED_TARGET libavcodec)
        pkg_check_modules(AVFORMAT   REQUIRED IMPORTED_TARGET libavformat)
        pkg_check_modules(AVUTIL     REQUIRED IMPORTED_TARGET libavutil)
        pkg_check_modules(SWSCALE    REQUIRED IMPORTED_TARGET libswscale)
        pkg_check_modules(SWRESAMPLE REQUIRED IMPORTED_TARGET libswresample)
        set(FFMPEG_TARGETS
            PkgConfig::AVCODEC PkgConfig::AVFORMAT PkgConfig::AVUTIL
            PkgConfig::SWSCALE PkgConfig::SWRESAMPLE)
    else()
        find_path(AVCODEC_INCLUDE_DIR libavcodec/avcodec.h REQUIRED)
        find_library(AVCODEC_LIBRARY    avcodec    REQUIRED)
        find_library(AVFORMAT_LIBRARY   avformat   REQUIRED)
        find_library(AVUTIL_LIBRARY     avutil     REQUIRED)
        find_library(SWSCALE_LIBRARY    swscale    REQUIRED)
        find_library(SWRESAMPLE_LIBRARY swresample REQUIRED)
        set(FFMPEG_TARGETS
            ${AVCODEC_LIBRARY} ${AVFORMAT_LIBRARY} ${AVUTIL_LIBRARY}
            ${SWSCALE_LIBRARY} ${SWRESAMPLE_LIBRARY})
    endif()
    message(STATUS "FFmpeg: ENABLED")
else()
    message(STATUS "FFmpeg: DISABLED (UI-only build)")
endif()

# spdlog
find_package(spdlog CONFIG QUIET)
if(NOT spdlog_FOUND)
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(SPDLOG IMPORTED_TARGET spdlog)
    if(SPDLOG_FOUND)
        set(SPDLOG_TARGET PkgConfig::SPDLOG)
    else()
        # Header-only fallback — spdlog bundled via FetchContent
        include(FetchContent)
        FetchContent_Declare(spdlog
            GIT_REPOSITORY https://github.com/gabime/spdlog.git
            GIT_TAG        v1.14.1
        )
        FetchContent_MakeAvailable(spdlog)
        set(SPDLOG_TARGET spdlog::spdlog)
    endif()
else()
    set(SPDLOG_TARGET spdlog::spdlog)
endif()

add_subdirectory(libs/libvidcut)
add_subdirectory(src)
