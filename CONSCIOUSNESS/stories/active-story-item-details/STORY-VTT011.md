# STORY-VTT011: PPA ships the Rust binary

## Context
The Debian packaging at `debian/rules` line 14 calls `$(MAKE) -f Makefile.linux`, which compiles the legacy C sources in `src/linux/*.c` and `src/common/*.c` using gcc. The PPA at `ppa:powellclark/voice-to-text` has been shipping this C binary for every release since 1.0.0 through 1.0.16, despite the Rust rewrite landing on 2026-04-07 in commit `2c26081`. The `debian/changelog` for 1.0.16 explicitly acknowledges: *"Rust rewrite added (not yet used for PPA build)"*. Ubuntu PPA users have never run the Rust binary.

Additionally, `debian/control` Depends lists `cmake`, `g++`, `make`, `python3 (>= 3.10)`, `python3-pip` — build tools and a Python runtime stack needed only for the legacy C implementation. After STORY-VTT010 removes Python, these become dead dependencies that bloat installations.

## Acceptance Criteria
1. `debian/rules` `override_dh_auto_build` invokes `cargo build --release` instead of `$(MAKE) -f Makefile.linux`
2. `debian/rules` `override_dh_auto_install` copies `target/release/vtt-linux` to `debian/voice-to-text/usr/bin/vtt-linux` with mode 0755
3. `debian/control` Build-Depends list removes `portaudio19-dev`, `libx11-dev`, `libxtst-dev`, `libxext-dev` (replaced by Rust crate build dependencies) and adds `rustc (>= 1.82)`, `cargo (>= 1.82)`, `libclang-dev`, `libssl-dev`, `pkg-config`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `libnotify-dev`, `libasound2-dev`, `libx11-dev`, `libxtst-dev`, `libxext-dev`, `libvulkan-dev`
4. `debian/control` Depends removes `python3`, `python3-pip`, `cmake`, `g++`, `make` and retains only the runtime shared libraries needed by the Rust binary
5. A `debian/postinst` script downloads `ggml-small.en.bin` (~244 MB) to `/usr/share/voice-to-text/models/` on first install if not already present, with sha256 verification and a graceful failure message if network is unavailable
6. `debian/changelog` has a new `2.0.0` entry with one bullet per major change: "Rust rewrite now shipped — previous releases installed the legacy C binary despite rewrite being in the tree", "whisper-rs in-process replaces CT2 Python subprocess", "Python runtime dependency removed", "Default model downloaded on first install"
7. The package builds cleanly with `debuild -S -sa` and `sbuild --dist=noble` (or pbuilder) without warnings
8. After `sudo apt install ./voice-to-text_2.0.0_amd64.deb`, the installed binary at `/usr/bin/vtt-linux` matches the sha256 of `target/release/vtt-linux`
9. Running the installed binary from a fresh VM launches successfully, downloads the default model if absent, and transcribes a test recording end-to-end
10. Acceptance scenario: on a fresh Ubuntu 24.04 VM with the PPA added, `sudo apt install voice-to-text` installs the 2.0.0 package, the tray icon appears, the postinst has cached the default model, pressing Scroll Lock records and transcribes correctly

## Dependencies
- STORY-VTT010 must be complete and working locally before this story can be validated
- Launchpad PPA account `powellclark/voice-to-text` with GPG signing key

## Test Strategy
Build locally with `debuild -S -sa`; inspect the resulting `.deb` contents (`dpkg -c`) to confirm no Python files, no C source, no symlinks to legacy paths. Install in a fresh Ubuntu 24.04 Multipass VM; verify runtime dependencies resolve via `apt install` alone. Execute a transcription end-to-end. Only after local validation passes, `dput powellclark-voice-to-text ../voice-to-text_2.0.0_source.changes`.
