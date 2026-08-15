# 3. whisper-rs in-process model loading replaces CT2 Python subprocess

Date: 2026-04-17

## Status

Accepted

## Context

The current transcription pipeline spawns a fresh `python3 /usr/share/voice-to-text/transcribe.py` subprocess for every push-to-talk press. The Python process imports `faster_whisper`, loads the CTranslate2 model from disk into VRAM, performs inference, then exits. The Rust `transcribe_audio` function blocks on `cmd.output()` until this full cycle completes.

Observed symptoms on 2026-04-14 through 2026-04-17:

- Cold-load of `distil-large-v3.5` took 3+ minutes, exceeding the 90-second timeout.
- `large-v3-turbo` fell through the Rust match statement and triggered a repeated 1.6 GB download from the wrong HuggingFace repo (Systran vs mobiuslabsgmbh cached locally).
- Worker thread blocked indefinitely on stuck downloads; subsequent recordings queued in an mpsc channel and were silently lost.
- Log files confirmed a clean 2-second end-to-end latency only when the model file stayed resident in the OS page cache; eviction or new-model-selection produced multi-minute stalls.

The user's expectation — "the model was always in memory until you broke it" — is not currently met. Investigation of the pre-rewrite C codebase (`src/linux/transcribe.c:182-205, 256-277`) confirms the old binary also subprocess-ed per press; apparent persistence was incidental OS page caching of frequently-used small models.

The `debian/changelog` entry for 1.0.16 acknowledges: *"Rust rewrite added (not yet used for PPA build)"*. The PPA has been shipping the pre-rewrite C binary since the rewrite landed on 2026-04-07.

## Decision

Replace the Python subprocess transcription path with `whisper-rs 0.16` executed in-process within the Rust application. Use Vulkan as the GPU backend on Linux and Windows (universal NVIDIA + AMD + Intel support without requiring the CUDA Toolkit) and Metal on macOS (covers both Intel Radeon and Apple Silicon).

Architectural shape:

- A dedicated Rust worker thread owns a `WhisperEngine` struct wrapping `WhisperContext` and `WhisperState`.
- The engine loads the selected GGML model once at application startup and retains it for the lifetime of the process.
- Model switching is handled by dropping the current context and constructing a new one on the worker thread; the tray emits a `Loading model…` status during the switch.
- Audio samples flow from `audio.rs` as `&[f32]` at 16 kHz mono directly to `WhisperState::full()`; the WAV file round-trip is retained only for the debug recordings archive.
- The worker thread is restart-safe: if `whisper-rs` panics, the thread catches, logs, and re-spawns with the same model.

The CTranslate2 / Python path is removed entirely. `src/common/transcribe.py`, `python3` as a runtime dependency, `pip install faster-whisper ctranslate2`, and all CT2-related menu entries are deleted.

GGML models are downloaded on demand from `https://huggingface.co/ggerganov/whisper.cpp` to `~/.cache/voice-to-text/models/`. The Debian postinst script pre-downloads `ggml-small.en.bin` (~244 MB) so first-run transcriptions succeed without network access.

Prior art: `cjpais/Handy` (MIT-licensed, Rust, cross-platform) validates this architecture in production using `transcribe-rs` (a `whisper-rs` wrapper) with the same Vulkan/Metal feature split. We reuse the architectural choice and improve on it by keeping native GTK on Linux (avoiding the Tauri browser engine overhead), using `whisper-rs` directly (one less abstraction), and routing raw f32 samples from `cpal` to the engine without WAV intermediate files.

## Consequences

### Positive

- Sub-second transcription for typical push-to-talk clips regardless of model size.
- No Python runtime dependency; `debian/control` Depends list loses `python3`, `python3-pip`, `cmake`, `g++`, `make`, and gains only `rustc`, `cargo`, `libclang-dev`.
- Cross-platform parity: the same binary behaves identically on Linux, macOS (Intel + Apple Silicon), and Windows.
- Apple Silicon gains access to CoreML/Metal via whisper.cpp's native support, which CT2 cannot match.
- Packaging is dramatically simplified: a single binary and one GGML file per installed model.
- Worker-owned engine enables future enhancements (streaming partial results, VAD integration, model warm-swap).

### Negative

- Raw inference on NVIDIA GPUs is approximately 30-50% slower per forward pass than CTranslate2. Masked by the removal of the spawn + load tax, which dominated end-to-end latency.
- First `cargo build --release` takes ~60 seconds longer because `whisper-rs-sys` compiles bundled whisper.cpp from source. One-time cost per machine; cached afterward.
- Requires `libclang-dev` at build time for the whisper.cpp bindgen step.
- Users with existing CT2 model caches (`~/.cache/huggingface/hub/models--*`) must re-download in GGML format. A one-time migration cost of ~244 MB for the default model; user-triggered downloads for larger models.

### Neutral

- Model quality is identical: GGML and CT2 are different serialisations of the same OpenAI Whisper weights. Users hear no accuracy change.
- The `vtt.service` systemd user service continues to work unchanged; only the binary contents differ.

## Rollback

All changes land on branch `whisper-rs-migration`. If whisper-rs proves unsuitable (quality regression for British English plus programming vocabulary is the primary risk), revert with `git branch -D whisper-rs-migration` and instead implement the persistent-Python-daemon alternative: a long-running `transcribe_daemon.py` process started at app launch, keeping the CT2 model resident, communicating with Rust over stdin/stdout line-delimited JSON. This preserves CT2 speed at the cost of continuing to ship Python.

## References

- ADR 0000 — Use Architecture Decision Records
- `cjpais/Handy` — https://github.com/cjpais/Handy (MIT, Rust, cross-platform voice-to-text)
- `tazz4843/whisper-rs` — https://github.com/tazz4843/whisper-rs (Rust bindings for whisper.cpp)
- `ggerganov/whisper.cpp` — https://github.com/ggerganov/whisper.cpp (upstream C++ implementation)
