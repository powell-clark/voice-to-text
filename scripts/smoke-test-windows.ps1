<#
.SYNOPSIS
  One-command Windows build + smoke-test for voice-to-text (TASK-VTT082).

.DESCRIPTION
  Run this from the repo root in PowerShell on a Windows x86-64 machine:

      powershell -ExecutionPolicy Bypass -File scripts\smoke-test-windows.ps1

  It pulls latest main, builds the release binary, confirms it exists, then
  prints the manual checklist a human must verify (record -> transcribe ->
  inject). Build is automated; the perceptual checks are not — VTT needs a
  microphone and a focused window, so the last steps are yours to confirm.

  Prerequisites (install once):
    - Rust (https://rustup.rs)  -> rustc, cargo on PATH
    - Visual Studio Build Tools with the C++ workload (MSVC linker)
    - LLVM/Clang (https://github.com/llvm/llvm-project/releases) for bindgen
      -> set LIBCLANG_PATH if cargo can't find libclang
#>

$ErrorActionPreference = "Stop"

Write-Host "== voice-to-text Windows smoke test (TASK-VTT082) ==" -ForegroundColor Cyan

# 1. Latest code
Write-Host "`n[1/4] git pull origin main" -ForegroundColor Yellow
git pull origin main

# 2. Build release
Write-Host "`n[2/4] cargo build --release" -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Host "BUILD FAILED" -ForegroundColor Red; exit 1 }

# 3. Confirm binary
$bin = "target\release\vtt.exe"
Write-Host "`n[3/4] checking $bin" -ForegroundColor Yellow
if (-not (Test-Path $bin)) { Write-Host "MISSING: $bin" -ForegroundColor Red; exit 1 }
$size = [math]::Round((Get-Item $bin).Length / 1MB, 1)
Write-Host "OK: $bin ($size MB)" -ForegroundColor Green

# 4. Manual checklist
Write-Host "`n[4/4] BUILD GREEN. Now verify by hand:" -ForegroundColor Green
@"
  1. Launch it:            .\$bin
  2. Tray icon appears in the system tray
  3. Default Whisper model downloads/loads on first run (watch tray status)
  4. Hold the push-to-talk hotkey, speak, release -> recording starts/stops
  5. Whisper transcribes (sub-second once the model is warm)
  6. Transcribed text injects into a focused app (Notepad / browser / terminal)
  7. Note the GPU path: Vulkan in use, or CPU fallback? (record what you see)

  File any Windows-specific defect with:  /consciousness:issue <what broke>
  Then mark TASK-VTT082 done when all boxes pass.
"@ | Write-Host
