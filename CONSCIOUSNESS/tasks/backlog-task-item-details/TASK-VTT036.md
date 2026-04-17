# TASK-VTT036: Update debian/control dependencies

## Context
`debian/control` Build-Depends and Depends reflect the old C build: gcc build tools, Python runtime, cmake. After switching to cargo, we need Rust toolchain at build time and only runtime shared libraries at install time.

## Acceptance Criteria
1. Build-Depends lists: `debhelper-compat (= 13)`, `rustc (>= 1.82)`, `cargo (>= 1.82)`, `libclang-dev`, `libvulkan-dev`, `pkg-config`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `libnotify-dev`, `libasound2-dev`, `libx11-dev`, `libxtst-dev`, `libxext-dev`
2. Depends lists: `${shlibs:Depends}`, `${misc:Depends}`, `libgtk-3-0`, `libayatana-appindicator3-1`, `libnotify4`, `libasound2`, `libx11-6`, `libxtst6`, `libxext6`, `libvulkan1` — no Python, no cmake, no g++, no make
3. Recommends: `vulkan-tools` (for debugging GPU availability) — optional, not required
4. Suggests: removed or trimmed (no more `cuda-toolkit-12-6` since Vulkan obviates the CUDA Toolkit requirement)
5. Description paragraph rewritten to reflect 2.0.0 reality: "Pure Rust voice-to-text with in-process Whisper inference. GPU-accelerated via Vulkan on Linux."
6. `sudo apt install voice-to-text` on a minimal Ubuntu Noble install pulls in only runtime libraries, not Rust or Python

## Technical Approach
Edit `debian/control` in place. Example:

```
Source: voice-to-text
Section: sound
Priority: optional
Maintainer: Emmanuel Powell-Clark <emmanuel@powellclark.com>
Build-Depends: debhelper-compat (= 13),
               rustc (>= 1.82),
               cargo (>= 1.82),
               libclang-dev,
               libvulkan-dev,
               pkg-config,
               libgtk-3-dev,
               libayatana-appindicator3-dev,
               libnotify-dev,
               libasound2-dev,
               libx11-dev,
               libxtst-dev,
               libxext-dev
Standards-Version: 4.6.2
Homepage: https://github.com/powell-clark/voice-to-text
...

Package: voice-to-text
Architecture: amd64
Depends: ${shlibs:Depends},
         ${misc:Depends},
         libgtk-3-0,
         libayatana-appindicator3-1,
         libnotify4,
         libasound2,
         libx11-6,
         libxtst6,
         libxext6,
         libvulkan1
Recommends: vulkan-tools
Description: Pure Rust voice-to-text with in-process Whisper inference
 ...
```

## Test Strategy
Build the package. Install on a clean Ubuntu Noble VM with `apt install ./voice-to-text_2.0.0_amd64.deb`. Confirm no Python packages are pulled in as dependencies via `apt-cache depends voice-to-text`. Confirm the binary runs with `/usr/bin/vtt-linux`.

## Files
- `debian/control` (rewrite)
