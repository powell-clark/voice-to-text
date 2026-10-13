# whisper-rs-sys 0.15.0, patched for voice-to-text

A copy of [`whisper-rs-sys`](https://codeberg.org/tazz4843/whisper-rs) 0.15.0
(Unlicense, public domain), the raw FFI crate under `whisper-rs` 0.16. The root
`Cargo.toml` swaps it in for the crates.io crate through `[patch.crates-io]`, so
`whisper-rs` builds unchanged on top of it. Why: ADR-0011.

What differs from upstream, all in `build.rs` and `wrapper.h`:

1. It builds `../whisper.cpp` — the submodule pinned to a whisper.cpp release —
   instead of a copy bundled inside the crate. `WHISPER_CPP_DIR` points it
   elsewhere. A missing submodule stops the build with
   `git submodule update --init`.
2. It reads the version from whisper.cpp >= 1.9's `WHISPER_VERSION_*` variables
   (the upstream script panics on them) and builds with
   `WHISPER_BUILD_IS_DEV=OFF`, so `whisper_version()` reports the release.
3. It links `libparakeet` and binds `include/parakeet.h`, so the Parakeet
   engine's raw API is generated from the header it was compiled from.
4. Under the `vulkan` feature it installs `../SPIRV-Headers` (pinned submodule,
   or `SPIRV_HEADERS_DIR`) into `OUT_DIR` for ggml-vulkan, which requires it
   from ggml 0.26.
5. bindgen failing is an error. Upstream falls back to a bundled `bindings.rs`
   that matches an older whisper.cpp; this copy has none.

To move the engine: check out the new whisper.cpp tag in `../whisper.cpp`,
rebuild, run the tests and `vtt --version`, and record the move in the
CHANGELOG.
