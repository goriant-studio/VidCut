set(SOURCES
    main.cpp
    ui/MainWindow.cpp
    ui/TimelineWidget.cpp
    ui/PreviewWidget.cpp
    ui/MediaBrowserWidget.cpp
    ui/InspectorWidget.cpp
    ui/ToolBar.cpp
    ui/ExportDialog.cpp
    media/ThumbnailGenerator.cpp
    media/ExportEncoder.cpp
    media/AudioDecoder.cpp
)

# FFmpeg-linked decoder — only when explicitly enabled
if(VIDCUT_ENABLE_FFMPEG)
    list(APPEND SOURCES media/MediaDecoder.cpp)
endif()

set(RESOURCES ../resources/app.qrc)

# Windows resource file (.rc)
if(WIN32)
    list(APPEND SOURCES ../resources/vidcut.rc)
endif()

add_executable(VidCut WIN32 ${SOURCES} ${RESOURCES})

target_link_libraries(VidCut
    PRIVATE
        libvidcut
        Qt6::Core
        Qt6::Widgets
        Qt6::Multimedia
        Qt6::MultimediaWidgets
        ${FFMPEG_TARGETS}
        ${SPDLOG_TARGET}
)

target_include_directories(VidCut
    PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}
)

# Include FFmpeg headers when using the manual find_library fallback
if(DEFINED AVCODEC_INCLUDE_DIR)
    target_include_directories(VidCut PRIVATE ${AVCODEC_INCLUDE_DIR})
endif()

# MinGW: link against required system libraries
if(MINGW)
    target_link_libraries(VidCut PRIVATE -mwindows)
endif()

