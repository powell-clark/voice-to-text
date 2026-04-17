# FEAT-VTT025: Metal GPU acceleration on macOS

## Kano
performance (p1)

## Description
Whisper inference on macOS runs on the Apple Metal GPU backend. This covers both the user's 2019 Intel i9 Mac (AMD Radeon Pro 5500M) and any Apple Silicon Mac. Metal was chosen over Vulkan because Apple platforms do not ship Vulkan; whisper.cpp's bundled Metal backend is the native path.

## User Observable Behaviour
- On the 2019 Intel i9 Mac, launching VTT shows the Metal GPU in use (via Activity Monitor → GPU tab or `Metal Performance HUD`)
- Transcription of a 5-second clip on the Radeon Pro 5500M completes in under 600 ms
- On Apple Silicon (M1/M2/M3), transcription is dramatically faster — a 5-second clip completes in under 250 ms
- No CUDA Toolkit, no Python, no Rosetta is required
- The VTT .app bundle contains no CUDA libraries, no Python, no non-Rust binaries beyond whisper.cpp's compiled static library

## Acceptance Criteria
1. `Cargo.toml` `[target.'cfg(target_os = "macos")'.dependencies]` declares `whisper-rs = { version = "0.16", features = ["metal"] }`
2. The macOS .app bundle built via `cargo bundle` runs on macOS 12 Monterey and later
3. On 2019 Intel Mac: transcription of 5-second clip under 600 ms end-to-end after model warm
4. On Apple Silicon: transcription of 5-second clip under 250 ms end-to-end after model warm
5. Info.plist declares `NSMicrophoneUsageDescription` and accessibility usage strings; first run prompts for mic permission and accessibility
6. `lipo -info /Applications/VTT.app/Contents/MacOS/vtt-linux` confirms the target architecture (x86_64 for Intel, arm64 for Apple Silicon, or universal for a fat binary)

## Linked Tasks
- TASK-VTT025

## Parent Story
- STORY-VTT010 (MVP; per-platform polishing deferred to STORY-VTT012)
