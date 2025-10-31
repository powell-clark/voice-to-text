# Voice-to-Text Windows 11 Port - Complete Implementation Guide

## Status: Ready for Implementation

This document provides the complete implementation strategy for porting Voice-to-Text to Windows 11 with production-ready installer.

---

## Development Environment Setup

### Required Software

```powershell
# 1. Visual Studio 2022 Community (free)
# Download from: https://visualstudio.microsoft.com/vs/community/
# Required workloads:
#   - Desktop development with C++
#   - Python development (optional, for debugging)

# 2. vcpkg (Package Manager)
git clone https://github.com/microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg integrate install

# 3. Install PortAudio via vcpkg
.\vcpkg install portaudio:x64-windows

# 4. Python 3.12+ (for transcription backend)
# Download from: https://www.python.org/downloads/

# 5. Install Python dependencies
pip install faster-whisper ctranslate2 pyinstaller

# 6. Inno Setup (for installer creation)
# Download from: https://jrsoftware.org/isdl.php
```

---

## Build System: CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.20)
project(VoiceToText VERSION 1.0.0 LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find packages
find_package(portaudio CONFIG REQUIRED)

# Windows-specific: Add WIN32 to create GUI app (no console)
add_executable(vtt-windows WIN32
    src/windows/main.cpp
    src/windows/keyboard.cpp
    src/windows/typing.cpp
    src/windows/audio.cpp
    src/windows/gui.cpp
    src/common/settings.c
    src/common/logging.c
    src/common/queue.c
    src/windows/resources.rc  # Icon and version info
)

target_include_directories(vtt-windows PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
)

target_link_libraries(vtt-windows PRIVATE
    portaudio
    shell32  # For Shell_NotifyIcon
    user32   # For RegisterHotKey, SendInput
    advapi32 # For registry access
    ole32    # For COM initialization
    shlwapi  # For path functions
)

# Copy Python files to build directory
add_custom_command(TARGET vtt-windows POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_SOURCE_DIR}/src/common/transcribe.py
        $<TARGET_FILE_DIR:vtt-windows>/transcribe.py
)

# Set icon
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/assets/icon.ico")
    target_sources(vtt-windows PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/assets/icon.ico")
endif()

# Install targets
install(TARGETS vtt-windows DESTINATION bin)
install(FILES src/common/transcribe.py DESTINATION bin)
```

---

## Python Backend Bundling with PyInstaller

Create `build-python-bundle.py`:

```python
#!/usr/bin/env python3
"""
Bundle Python transcription backend into standalone executable
"""
import PyInstaller.__main__
import sys
import os

PyInstaller.__main__.run([
    'src/common/transcribe.py',
    '--onefile',
    '--name=vtt-transcribe',
    '--hidden-import=faster_whisper',
    '--hidden-import=ctranslate2',
    '--hidden-import=torch',
    '--collect-all=faster_whisper',
    '--collect-all=ctranslate2',
    '--noconfirm',
    '--windowed',  # No console window
])

print("\\n✅ Python bundle created: dist/vtt-transcribe.exe")
```

---

## Installer Script: Inno Setup

Create `installer/vtt-setup.iss`:

```pascal
#define MyAppName "Voice to Text"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Powell-Clark Limited"
#define MyAppURL "https://github.com/powell-clark/voice-to-text"
#define MyAppExeName "vtt-windows.exe"

[Setup]
AppId={{8F3D9C5A-1B2E-4D7C-9A3F-6E8D4C2A1B5E}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={autopf}\\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=installer\\output
OutputBaseFilename=VoiceToText-{#MyAppVersion}-win64-setup
SetupIconFile=assets\\icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "startup"; Description: "Start Voice to Text automatically on system boot"; GroupDescription: "Startup Options:"

[Files]
Source: "build\\Release\\vtt-windows.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\\vtt-transcribe.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\\*"; DestDir: "{app}\\assets"; Flags: ignoreversion recursesubdirs
Source: "README-WINDOWS.md"; DestDir: "{app}"; Flags: ignoreversion isreadme
; PortAudio DLL (if not statically linked)
Source: "C:\\vcpkg\\installed\\x64-windows\\bin\\portaudio.dll"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\\{#MyAppName}"; Filename: "{app}\\{#MyAppExeName}"
Name: "{group}\\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\\{#MyAppName}"; Filename: "{app}\\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; Auto-start on boot (HKCU, not HKLM - no admin required)
Root: HKCU; Subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Run"; ValueType: string; ValueName: "VoiceToText"; ValueData: """{app}\\{#MyAppExeName}"""; Flags: uninsdeletevalue; Tasks: startup

[Run]
Filename: "{app}\\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Kill process before uninstall
Filename: "taskkill"; Parameters: "/F /IM vtt-windows.exe"; Flags: runhidden; RunOnceId: "KillVTT"

[Code]
// Check if app is running before install
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
begin
  Result := True;
  if CheckForMutexes('VoiceToTextRunning') then
  begin
    if MsgBox('Voice to Text is currently running. Please close it before continuing.', mbError, MB_OKCANCEL) = IDOK then
    begin
      Exec('taskkill', '/F /IM vtt-windows.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
      Sleep(1000);
    end
    else
      Result := False;
  end;
end;

// First-run: Download default Whisper model
procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssPostInstall then
  begin
    if MsgBox('Would you like to download the default Whisper model now (small.en, ~250MB)?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      // Launch transcribe script with download flag
      Exec(ExpandConstant('{app}\\vtt-transcribe.exe'), '--download-model small.en', '', SW_SHOW, ewWaitUntilTerminated, ResultCode);
    end;
  end;
end;
```

---

## Complete Build Instructions

### From Windows (Native Build)

```batch
@echo off
REM Build Voice to Text for Windows

REM 1. Configure CMake
cmake -B build -S . -DCMAKE_TOOLCHAIN_FILE=C:/vcpkg/scripts/buildsystems/vcpkg.cmake

REM 2. Build Release
cmake --build build --config Release

REM 3. Bundle Python backend
python build-python-bundle.py

REM 4. Create installer
"C:\\Program Files (x86)\\Inno Setup 6\\ISCC.exe" installer\\vtt-setup.iss

echo.
echo ========================================
echo Build complete!
echo Installer: installer\\output\\VoiceToText-1.0.0-win64-setup.exe
echo ========================================
pause
```

### From Mac/Ubuntu (Cross-Compile)

```bash
#!/bin/bash
# Cross-compile for Windows from Linux/Mac

# Install MinGW-w64 cross-compiler
# Ubuntu: sudo apt install mingw-w64
# Mac: brew install mingw-w64

# Install dependencies via vcpkg (cross-compile triplet)
./vcpkg install portaudio:x64-mingw-static

# Configure CMake for cross-compilation
cmake -B build-windows -S . \\
    -DCMAKE_TOOLCHAIN_FILE=./vcpkg/scripts/buildsystems/vcpkg.cmake \\
    -DVCPKG_TARGET_TRIPLET=x64-mingw-static \\
    -DCMAKE_SYSTEM_NAME=Windows \\
    -DCMAKE_C_COMPILER=x86_64-w64-mingw32-gcc \\
    -DCMAKE_CXX_COMPILER=x86_64-w64-mingw32-g++

# Build
cmake --build build-windows --config Release

# Bundle Python (requires Windows or Wine)
echo "Note: Python bundling requires running on Windows or via Wine"
echo "Transfer dist/ folder to Windows machine for final installer creation"
```

---

## Testing on Your Windows 11 Machine

### Prerequisites Check

```batch
@echo off
echo Checking Windows 11 development environment...

REM Check Visual Studio
where cl.exe >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Visual Studio C++ compiler not found
    echo    Install Desktop Development with C++ workload
) else (
    echo ✅ Visual Studio C++ compiler found
)

REM Check vcpkg
if exist "C:\\vcpkg\\vcpkg.exe" (
    echo ✅ vcpkg found
) else (
    echo ❌ vcpkg not found at C:\\vcpkg
)

REM Check Python
where python >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Python not found
) else (
    echo ✅ Python found
    python --version
)

REM Check NVIDIA GPU (for CUDA acceleration)
where nvidia-smi >nul 2>&1
if %errorlevel% neq 0 (
    echo ⚠️  NVIDIA GPU not detected (CPU transcription will be used)
) else (
    echo ✅ NVIDIA GPU detected
    nvidia-smi --query-gpu=name --format=csv,noheader
)

echo.
echo Check your CUDA installation for RTX 2060 SUPER:
nvidia-smi

pause
```

### Manual Test Run

```batch
@echo off
REM Run Voice to Text directly from build directory

cd build\\Release

REM Set environment for testing
set VTT_LOG_LEVEL=DEBUG
set VTT_CONFIG_DIR=%TEMP%\\vtt-test

REM Run
vtt-windows.exe

REM Check logs
type %VTT_CONFIG_DIR%\\vtt.log
```

---

## Expected Installation Size

```
Program Files\\Voice to Text\\
├── vtt-windows.exe           (~2 MB - C++ binary)
├── vtt-transcribe.exe        (~150 MB - PyInstaller bundle with faster-whisper)
├── portaudio.dll             (~200 KB - if dynamically linked)
├── transcribe.py             (~10 KB - backup script)
└── assets\\
    └── icon.ico              (~50 KB)

%USERPROFILE%\\.cache\\whisper\\  (model storage, downloaded on demand)
└── small.en.pt               (~250 MB - default model)

Total: ~400 MB installed + 250 MB default model = ~650 MB
```

---

## Auto-Start Configuration

The installer adds registry key:

```
HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run
Key: VoiceToText
Value: "C:\\Program Files\\Voice to Text\\vtt-windows.exe"
```

Users can disable auto-start via:
- Windows Settings → Apps → Startup
- Or uncheck during installation

---

## CUDA Acceleration Setup (Your RTX 2060 SUPER)

The Python backend will automatically detect CUDA if installed:

```batch
@echo off
REM Verify CUDA installation for GPU acceleration

REM 1. Check CUDA Toolkit
nvcc --version

REM 2. Check cuDNN (required for CTranslate2)
where cudnn64_*.dll

REM 3. Test CTranslate2 GPU support
python -c "import ctranslate2; print(f'CUDA devices: {ctranslate2.get_cuda_device_count()}')"

REM Expected output for RTX 2060 SUPER:
REM CUDA devices: 1
```

If CUDA not detected:
1. Install CUDA Toolkit 12.x from NVIDIA
2. Install cuDNN 8.x
3. Restart after installation

---

## Distribution Checklist

- [ ] Build Windows executable (vtt-windows.exe)
- [ ] Bundle Python backend (vtt-transcribe.exe)
- [ ] Create icon file (assets/icon.ico)
- [ ] Compile Inno Setup installer
- [ ] Test on clean Windows 11 VM
- [ ] Test with/without CUDA
- [ ] Test microphone detection (built-in + USB)
- [ ] Test hotkey (Scroll Lock + custom keys)
- [ ] Test text injection (Notepad, Word, Slack, VS Code)
- [ ] Test auto-start registry key
- [ ] Test uninstaller
- [ ] Create README-WINDOWS.md
- [ ] Upload to GitHub Releases
- [ ] Create winget manifest (optional)
- [ ] Submit to Chocolatey (optional)

---

## Troubleshooting

### Issue: "PortAudio DLL not found"

**Solution:** Statically link PortAudio in CMake:

```cmake
find_package(portaudio CONFIG REQUIRED)
target_link_libraries(vtt-windows PRIVATE portaudio_static)
```

### Issue: "Python backend fails to start"

**Solution:** Check PyInstaller bundle includes all dependencies:

```bash
python -m PyInstaller --onefile --collect-all faster_whisper --collect-all ctranslate2 transcribe.py
```

### Issue: "Hotkey not working"

**Solution:** Ensure no other app has registered the same key. Try F12 as alternative:

```cpp
keyboard.hotkey_vk = VK_F12;
keyboard.hotkey_modifiers = 0;
```

### Issue: "SendInput text appears garbled"

**Solution:** Use clipboard paste fallback for Unicode:

```cpp
// In typing.cpp, force clipboard mode for all text
paste_text(text);
```

---

## Next Steps for You

1. **Set up Windows dev environment** (Visual Studio, vcpkg, Python)
2. **I'll create the remaining source files** (gui.cpp, main.cpp)
3. **Build on your Windows machine** using provided CMakeLists.txt
4. **Test with your RTX 2060 SUPER** (CUDA acceleration)
5. **Create installer** using Inno Setup script
6. **Deploy** to GitHub Releases

Would you like me to continue creating the remaining source files (gui.cpp and main.cpp)? These are the largest files (~1500 lines combined) and will complete the Windows port.
