Third-party code built into Voice to Text (ADR-0011).

whisper.cpp (https://github.com/ggml-org/whisper.cpp) — submodule, MIT
- Pinned to a release tag; `vtt --version` prints which.
- Provides both speech engines: Whisper (libwhisper) and Parakeet
  (libparakeet), on the ggml runtime with Vulkan, Metal or CPU backends.

SPIRV-Headers (https://github.com/KhronosGroup/SPIRV-Headers) — submodule
- Header-only. Needed by ggml's Vulkan backend at build time only.

whisper-rs-sys — patched copy of the crate, Unlicense
- Builds the whisper.cpp submodule and generates the Rust bindings for both
  engines. See whisper-rs-sys/README.md for what differs from upstream.

Fetch the submodules before the first build:

    git submodule update --init

Model files are not kept here. They download on first use into the user's
cache directory (or are pre-installed by the Debian package).
