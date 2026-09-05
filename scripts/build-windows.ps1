<#
.SYNOPSIS
  Reproducible Windows release build for voice-to-text.

.DESCRIPTION
  whisper-rs-sys has two native build-time dependencies that GitHub's
  windows-latest runners ship preinstalled but a fresh dev machine does not:

    1. libclang  — bindgen generates the whisper.cpp FFI bindings
    2. cmake     — compiles the bundled whisper.cpp static library

  This script discovers both from what's already on a typical Windows dev box
  (Visual Studio 2022 Build Tools bundle cmake + ninja; the PyPI `libclang`
  wheel ships libclang.dll), wires LIBCLANG_PATH and PATH, then builds. It does
  not download a multi-hundred-MB LLVM toolchain just to get one DLL.

  Run from the repo root:
      powershell -ExecutionPolicy Bypass -File scripts\build-windows.ps1

  Prerequisites (install once if discovery fails):
    - Rust (https://rustup.rs)
    - Visual Studio 2022 Build Tools with the "Desktop development with C++"
      workload (provides MSVC, cmake, ninja)
    - libclang: `pip install libclang`  (or install full LLVM)
#>
$ErrorActionPreference = "Stop"

function Find-First($paths) {
  foreach ($p in $paths) { if ($p -and (Test-Path $p)) { return $p } }
  return $null
}

Write-Host "== voice-to-text Windows build ==" -ForegroundColor Cyan

# --- cargo -------------------------------------------------------------------
$cargoBin = Find-First @("$env:USERPROFILE\.cargo\bin\cargo.exe")
if (-not $cargoBin) {
  if (Get-Command cargo -ErrorAction SilentlyContinue) { $cargoBin = (Get-Command cargo).Source }
}
if (-not $cargoBin) { throw "cargo not found. Install Rust from https://rustup.rs" }
$env:Path = "$(Split-Path $cargoBin);$env:Path"
Write-Host "cargo:    $cargoBin" -ForegroundColor Green

# --- cmake (PATH, then VS bundle) -------------------------------------------
$cmake = $null
if (Get-Command cmake -ErrorAction SilentlyContinue) { $cmake = (Get-Command cmake).Source }
if (-not $cmake) {
  $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
  if (Test-Path $vswhere) {
    $vs = & $vswhere -products * -property installationPath | Select-Object -First 1
    if ($vs) {
      $cand = Join-Path $vs "Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
      if (Test-Path $cand) { $cmake = $cand }
    }
  }
}
if (-not $cmake) { throw "cmake not found. Install the C++ workload in VS Build Tools, or Kitware.CMake." }
$env:Path = "$(Split-Path $cmake);$env:Path"
Write-Host "cmake:    $cmake" -ForegroundColor Green

# --- libclang ----------------------------------------------------------------
if (-not $env:LIBCLANG_PATH -or -not (Test-Path (Join-Path $env:LIBCLANG_PATH "libclang.dll"))) {
  $candidates = @(
    "$env:ProgramFiles\LLVM\bin\libclang.dll",
    "$env:APPDATA\Python\Python314\site-packages\clang\native\libclang.dll",
    "$env:APPDATA\Python\Python313\site-packages\clang\native\libclang.dll"
  )
  # Also probe the active Python's user site for the libclang wheel.
  $py = Get-Command python -ErrorAction SilentlyContinue
  if ($py) {
    try {
      $usite = & $py.Source -c "import site; print(site.getusersitepackages())"
      if ($usite) { $candidates += (Join-Path $usite "clang\native\libclang.dll") }
    } catch {}
  }
  $dll = Find-First $candidates
  if (-not $dll) {
    throw "libclang.dll not found. Run: pip install libclang  (or install full LLVM)."
  }
  $env:LIBCLANG_PATH = Split-Path $dll
}
Write-Host "libclang: $env:LIBCLANG_PATH" -ForegroundColor Green

# --- Vulkan SDK (GPU acceleration, FEAT-VTT024) ------------------------------
# whisper-rs builds with the `vulkan` feature on Windows. The SDK provides glslc
# (compiles the GGML compute shaders) + headers + vulkan-1.lib. The runtime
# vulkan-1.dll ships with the GPU driver.
if (-not $env:VULKAN_SDK -or -not (Test-Path "$env:VULKAN_SDK\Bin\glslc.exe")) {
  $sdkRoot = "C:\VulkanSDK"
  $sdk = $null
  if (Test-Path $sdkRoot) {
    $sdk = (Get-ChildItem $sdkRoot -Directory | Sort-Object Name -Descending |
            Where-Object { Test-Path "$($_.FullName)\Bin\glslc.exe" } |
            Select-Object -First 1).FullName
  }
  if (-not $sdk) {
    throw "Vulkan SDK not found. Install it (winget install KhronosGroup.VulkanSDK) " +
          "or from https://vulkan.lunarg.com/sdk/home#windows"
  }
  $env:VULKAN_SDK = $sdk
}
$env:Path = "$env:VULKAN_SDK\Bin;$env:Path"
Write-Host "vulkan:   $env:VULKAN_SDK" -ForegroundColor Green

# --- short target dir (MAX_PATH workaround) ----------------------------------
# whisper.cpp's Vulkan backend builds a nested `vulkan-shaders-gen` sub-project.
# Under the long default target path the MSBuild FileTracker .tlog paths exceed
# Windows' 260-char MAX_PATH and fail with FTK1011. A short CARGO_TARGET_DIR
# keeps every generated path under the limit.
if (-not $env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = "C:\vtt" }
Write-Host "target:   $env:CARGO_TARGET_DIR" -ForegroundColor Green

# --- build -------------------------------------------------------------------
Write-Host "`ncargo build --release" -ForegroundColor Yellow
& cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Host "BUILD FAILED ($LASTEXITCODE)" -ForegroundColor Red; exit $LASTEXITCODE }

$bin = "target\release\vtt.exe"
if (-not (Test-Path $bin)) { Write-Host "MISSING: $bin" -ForegroundColor Red; exit 1 }
$size = [math]::Round((Get-Item $bin).Length / 1MB, 1)
Write-Host "`nOK: $bin ($size MB)" -ForegroundColor Green
