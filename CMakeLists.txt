cmake_minimum_required(VERSION 3.20)
project(VoiceToText VERSION 1.0.5 LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_C_STANDARD 11)

# Platform detection
if(WIN32)
    message(STATUS "Building for Windows")

    # Find packages
    find_package(portaudio CONFIG REQUIRED)

    # Common source files
    set(COMMON_SOURCES
        src/common/settings.c
        src/common/logging.c
        src/common/queue.c
    )

    # Windows-specific sources
    set(WINDOWS_SOURCES
        src/windows/main.cpp
        src/windows/keyboard.cpp
        src/windows/typing.cpp
        src/windows/audio.cpp
        src/windows/gui.cpp
    )

    # Create Windows GUI executable (no console window)
    add_executable(vtt-windows WIN32
        ${COMMON_SOURCES}
        ${WINDOWS_SOURCES}
    )

    target_include_directories(vtt-windows PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
    )

    target_link_libraries(vtt-windows PRIVATE
        portaudio
        shell32    # Shell_NotifyIcon
        user32     # RegisterHotKey, SendInput
        advapi32   # Registry access
        ole32      # COM initialization
        shlwapi    # Path functions
        comctl32   # Common controls
    )

    # Copy transcribe.py to build directory
    add_custom_command(TARGET vtt-windows POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy
            ${CMAKE_SOURCE_DIR}/src/common/transcribe.py
            $<TARGET_FILE_DIR:vtt-windows>/transcribe.py
        COMMENT "Copying transcribe.py to build directory"
    )

    # Install
    install(TARGETS vtt-windows DESTINATION bin)
    install(FILES src/common/transcribe.py DESTINATION bin)

elseif(UNIX AND NOT APPLE)
    # Linux build (existing Makefile.linux)
    message(STATUS "For Linux, use: make -f Makefile.linux")

elseif(APPLE)
    # macOS build (existing Makefile)
    message(STATUS "For macOS, use: make")

endif()
