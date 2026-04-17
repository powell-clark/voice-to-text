# STORY-VTT010: In-process Whisper model loaded persistently

## Context
Every push-to-talk press currently spawns a fresh `python3 /usr/share/voice-to-text/transcribe.py` subprocess that imports faster-whisper, loads the CTranslate2 model from disk into VRAM, and then exits. For the user's current model `CT2 large-v3-turbo`, the cold load exceeds 90 seconds on the first press and the 90-second timeout kills it; subsequent presses block on the mpsc channel because the worker thread is still stuck. The apparent "it used to work" behaviour relied entirely on incidental OS page caching of smaller models — when the user switched to larger models or the cache was evicted, the architecture failed completely.

ADR-0003 resolves this by loading the Whisper model once in a dedicated worker thread that lives for the lifetime of the application. Transcription becomes an in-process function call on pre-loaded state, eliminating both the subprocess spawn tax and the model reload cost.

## Acceptance Criteria
1. On application startup, a dedicated worker thread loads the GGML model named in `settings.conf` into a `whisper_rs::WhisperContext` and creates a `WhisperState`; the tray status reads `Loading model...` during this phase and transitions to `Ready` when loading completes
2. Each subsequent push-to-talk recording reuses the existing `WhisperContext` — no new subprocess is spawned, no model is re-loaded from disk, no Python interpreter is invoked
3. Average end-to-end press-to-text latency for a 5-second clip on the user's RTX 2060 SUPER Linux machine is under 500 milliseconds measured across ten consecutive presses
4. The transcription worker thread never blocks indefinitely — a transcription longer than 120 seconds triggers a warning log and returns an error to the UI without killing the engine
5. Switching models from the tray menu drops the current `WhisperContext`, emits `Loading model...` status, loads the new model, and returns `Ready` — the user's next press uses the new model with no restart required
6. Language switching (English ↔ Multilingual) on models that have both variants (`small`, `medium`) triggers a model reload; multilingual-only models (`large-v3-turbo`, `large-v3`) are unaffected
7. If the model file does not exist in `~/.cache/voice-to-text/models/`, the worker thread initiates download from `huggingface.co/ggerganov/whisper.cpp` with sha256 verification before loading; tray shows `Downloading model... X%`
8. If whisper-rs panics during inference, the worker thread catches the panic, logs it with the audio path, emits `Transcription error` status, and remains available for the next press — a single bad clip does not take down the engine
9. The binary `target/release/vtt-linux` contains no Python interpreter invocation, no `python3` string references in the transcription path, and does not link or load `libpython`
10. Acceptance scenario: restart VTT, press Scroll Lock for 3 seconds speaking "hello world test", release — transcribed text appears in the focused window in under 500 ms; repeat ten times with varying clip lengths — all complete in under 1 second; switch model to `medium` from the tray — `Loading model...` shown briefly, next press uses medium and transcribes in under 600 ms

## Dependencies
- ADR-0003 (whisper-rs in-process model) must be committed and accepted
- whisper-rs 0.16 crate with vulkan feature must build successfully on Linux with libclang-dev installed

## Rollback
The branch `whisper-rs-migration` contains all changes. If the transcription quality against British English programming vocabulary is measurably worse than CT2 baseline (defined as more than 10% additional word error rate on the user's existing recordings archive), revert with `git checkout main && git branch -D whisper-rs-migration` and pursue STORY-VTT017 (optional CT2 persistent Python daemon) as the alternative.
