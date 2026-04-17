# FEAT-VTT024: Vulkan GPU acceleration on Linux and Windows

## Kano
performance (p1)

## Description
Whisper inference on Linux and Windows runs on the user's GPU via Vulkan, regardless of vendor (NVIDIA, AMD, Intel). Vulkan was chosen over CUDA to avoid the large CUDA Toolkit dependency at install time and to support users without NVIDIA hardware without a separate build variant.

## User Observable Behaviour
- On the user's Linux machine with RTX 2060 SUPER, `nvidia-smi` during transcription shows GPU utilisation spiking to ~70-90% and VRAM usage ~2 GB during inference
- Transcription latency of a 5-second clip completes in under 400 ms after model is warm
- The installed package does not require `cuda-toolkit-12-6` to function
- The log at startup reads `GPU: NVIDIA GeForce RTX 2060 SUPER (Vulkan)` or similar
- On machines without GPU or with Vulkan drivers missing, whisper-rs falls back to CPU inference with no crash; the log reads `GPU acceleration unavailable, using CPU`

## Acceptance Criteria
1. `Cargo.toml` `[target.'cfg(target_os = "linux")'.dependencies]` and `[target.'cfg(target_os = "windows")'.dependencies]` declare `whisper-rs = { version = "0.16", features = ["vulkan"] }`
2. The release binary on Linux links against `libvulkan.so.1` (`ldd target/release/vtt-linux | grep vulkan`)
3. On NVIDIA hardware: transcription of a 5-second clip completes in under 500 ms end-to-end after model warm
4. On AMD or Intel iGPU hardware: transcription completes using Vulkan (verifiable by `vulkaninfo` output matching the detected adapter)
5. `debian/control` includes `libvulkan1` in Depends and `libvulkan-dev` in Build-Depends
6. `debian/control` Recommends `vulkan-tools` for optional user-facing diagnostics (`vulkaninfo`)

## Linked Tasks
- TASK-VTT025

## Parent Story
- STORY-VTT010
