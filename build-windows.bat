@echo off
REM Voice to Text - Windows Build Script
REM Builds production-ready installer for Windows 11

echo ========================================
echo Voice to Text - Windows Builder
echo ========================================
echo.

REM Check for Visual Studio
where cl.exe >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Visual Studio C++ compiler not found
    echo Please run this from "x64 Native Tools Command Prompt for VS 2022"
    pause
    exit /b 1
)

REM Check for vcpkg
if not exist "C:\vcpkg\vcpkg.exe" (
    echo ERROR: vcpkg not found at C:\vcpkg
    echo Install vcpkg: https://vcpkg.io/en/getting-started.html
    pause
    exit /b 1
)

REM Check for Python
where python >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Python not found
    echo Install Python 3.12+: https://www.python.org/downloads/
    pause
    exit /b 1
)

echo [1/5] Installing dependencies via vcpkg...
C:\vcpkg\vcpkg install portaudio:x64-windows
if %errorlevel% neq 0 (
    echo ERROR: Failed to install PortAudio
    pause
    exit /b 1
)

echo.
echo [2/5] Configuring CMake...
cmake -B build -S . -DCMAKE_TOOLCHAIN_FILE=C:/vcpkg/scripts/buildsystems/vcpkg.cmake -DCMAKE_BUILD_TYPE=Release
if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    pause
    exit /b 1
)

echo.
echo [3/5] Building Release binary...
cmake --build build --config Release
if %errorlevel% neq 0 (
    echo ERROR: Build failed
    pause
    exit /b 1
)

echo.
echo [4/5] Bundling Python backend...
python -m pip install --upgrade pip pyinstaller faster-whisper ctranslate2
python -m PyInstaller --onefile --name=vtt-transcribe --windowed --hidden-import=faster_whisper --hidden-import=ctranslate2 --noconfirm src/common/transcribe.py
if %errorlevel% neq 0 (
    echo ERROR: Python bundling failed
    pause
    exit /b 1
)

echo.
echo [5/5] Creating installer...
if exist "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" (
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\vtt-setup.iss
    if %errorlevel% neq 0 (
        echo ERROR: Installer creation failed
        pause
        exit /b 1
    )
) else (
    echo WARNING: Inno Setup not found. Skipping installer creation.
    echo Download from: https://jrsoftware.org/isdl.php
)

echo.
echo ========================================
echo BUILD SUCCESSFUL!
echo ========================================
echo.
echo Outputs:
echo   Binary: build\Release\vtt-windows.exe
echo   Python: dist\vtt-transcribe.exe
if exist "installer\output\VoiceToText-1.0.0-win64-setup.exe" (
    echo   Installer: installer\output\VoiceToText-1.0.0-win64-setup.exe
)
echo.
echo To test: cd build\Release ^&^& vtt-windows.exe
echo.
pause
