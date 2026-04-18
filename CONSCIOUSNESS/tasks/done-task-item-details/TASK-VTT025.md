# TASK-VTT025: Add whisper-rs to Cargo.toml with per-platform features

## Context
The project currently has no Whisper inference dependency in Rust — transcription shells out to Python. This task adds `whisper-rs 0.16` with the correct feature flags per target platform: Vulkan on Linux and Windows (universal GPU support), Metal on macOS (both Intel and Apple Silicon). The crate version is bumped from 1.0.16 to 2.0.0 to signal the breaking architectural change.

## Acceptance Criteria
1. `Cargo.toml` has `version = "2.0.0"`
2. `[dependencies]` contains `whisper-rs = { version = "0.16", default-features = false }` at the common level with no features enabled
3. `[target.'cfg(target_os = "linux")'.dependencies]` adds `whisper-rs = { version = "0.16", features = ["vulkan"] }`
4. `[target.'cfg(target_os = "windows")'.dependencies]` adds `whisper-rs = { version = "0.16", features = ["vulkan"] }`
5. `[target.'cfg(target_os = "macos")'.dependencies]` adds `whisper-rs = { version = "0.16", features = ["metal"] }`
6. `cargo check --target x86_64-unknown-linux-gnu` succeeds on the user's Linux machine with `libclang-dev` and `libvulkan-dev` installed
7. `cargo build --release` completes without errors; first build compiles bundled whisper.cpp once (~60 s), subsequent builds reuse the cached artefact
8. `cargo tree --package whisper-rs` shows version 0.16.x resolved and no Python-related transitive dependencies
9. The `sha2` crate is added for model file integrity verification; the `reqwest` crate is added with `blocking` and `rustls-tls` features for model downloads (no OpenSSL system dep required)

## Technical Approach
Edit `Cargo.toml` at the project root. Retain existing dependencies (cpal, hound, arboard, notify-rust, chrono, anyhow, dirs, libc). Add `sha2 = "0.10"` and `reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls", "stream"] }` to the common dependencies block.

Per-platform `whisper-rs` declarations inherit the base version specifier and extend with features. Cargo's feature unification means the Linux target sees `vulkan`, the Windows target sees `vulkan`, the macOS target sees `metal`, and common code sees none.

## Test Strategy
`cargo build --release` on Linux with Vulkan SDK installed; binary size and linkage inspected via `ldd target/release/vtt-linux` to confirm `libvulkan.so.1` appears. `cargo check --target aarch64-apple-darwin` attempted with `SDKROOT` set to confirm macOS feature resolution compiles (full macOS build deferred to TASK-VTT040).

## Files
- `Cargo.toml` (modify)
- `Cargo.lock` (regenerated)
