# TASK-VTT088: Enable Vulkan GPU acceleration on the Windows build

## Context

Requested by Emmanuel (2026-06-26): CPU transcription on Windows is too slow on
a machine with an NVIDIA RTX 2060 SUPER. The v2.2.0 Windows cut shipped CPU-only
by deliberate choice (TASK-VTT063) to get it working first. This task completes
the Windows half of FEAT-VTT024 (Vulkan GPU acceleration) — Linux already ships
Vulkan; macOS uses Metal.

Vulkan (not CUDA) so one build accelerates every GPU vendor (NVIDIA/AMD/Intel)
without a CUDA Toolkit dependency or a separate NVIDIA-only build variant.

## Approach

- `Cargo.toml` Windows target: `whisper-rs = { ..., features = ["vulkan"] }`
- Build-time: Vulkan SDK on the machine (headers + `vulkan-1.lib` + `glslc`);
  cmake's FindVulkan locates it via `VULKAN_SDK`. Runtime `vulkan-1.dll` already
  ships with the GPU driver.
- CI: add a Vulkan SDK install step to the `build-windows-msi` job in
  `release.yml` so the released MSI is GPU-enabled.
- whisper-rs falls back to CPU when no Vulkan device is present (no crash).

## Acceptance criteria

- [x] `cargo build --release` with the `vulkan` feature succeeds on Windows
      (after the MAX_PATH fix — see below)
- [x] Startup log shows the GPU is used — `ggml_vulkan: 0 = NVIDIA GeForce RTX
      2060 SUPER (NVIDIA) … matrix cores: NV_coopmat2`, `whisper_backend_init_gpu:
      using Vulkan0 backend` — replacing the old `no GPU found`
- [~] A 5-second clip transcribes well under CPU time — GPU is confirmed engaged
      (Vulkan0 backend); a clean warm-latency benchmark wasn't isolated because the
      test harness reloads the model per run (pays one-time pipeline warmup). The
      long-running tray app loads once and stays warm, so real-world inference is
      GPU-fast. Precise warm benchmark deferred to in-app measurement.
- [x] No-GPU / missing-Vulkan path still falls back to CPU — whisper.cpp registers
      both Vulkan and CPU backends and selects CPU when no Vulkan device is present
- [x] `release.yml` Windows job installs the Vulkan SDK (LunarG silent installer)
      so the shipped MSI is GPU-accelerated
- [x] FEAT-VTT024 AC#1 satisfied for Windows (Cargo.toml Windows target declares
      `features = ["vulkan"]`)

## MAX_PATH note

The Vulkan build initially failed with MSBuild `FTK1011` — whisper.cpp's nested
`vulkan-shaders-gen` sub-build exceeded Windows' 260-char MAX_PATH under the long
default target dir. Fixed by building under a short `CARGO_TARGET_DIR` (`C:\vtt`
locally, `C:\t` in CI). `scripts/build-windows.ps1` now sets this automatically.

## Dependencies

- Feature: FEAT-VTT024
- Story: STORY-VTT010
- Directive: DIRECT-VTT004
- Builds on: TASK-VTT082 (Windows working, CPU)
