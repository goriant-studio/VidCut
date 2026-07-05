cmake_minimum_required(VERSION 3.25)

set(CMAKE_AUTOMOC ON)

add_library(libvidcut STATIC
    src/Project.cpp
    src/Timeline.cpp
    src/Clip.cpp
    src/Track.cpp
    src/MediaAsset.cpp
    src/CommandManager.cpp
    src/MediaPool.cpp
)

target_include_directories(libvidcut
    PUBLIC include
)

find_package(Qt6 REQUIRED COMPONENTS Core Multimedia)
target_link_libraries(libvidcut PUBLIC Qt6::Core Qt6::Multimedia)
