# TASK-VTT027: Rewrite transcription worker in main.rs

## Context
The current transcription worker at `src/main.rs:231-322` pulls `WorkItem::Audio(PathBuf)` from a channel and calls `transcribe::transcribe_audio(&audio_path, &model, &language, &prompt)` which internally spawns `python3 transcribe.py`. The worker has no persistent engine state and reloads the model implicitly on every press via subprocess cold-start.

This task rewrites the worker to own a `WhisperEngine` for its lifetime, handle model-switch messages from the tray, and process transcription messages using the engine in-process. The subprocess spawn and the `transcribe::transcribe_audio` helper disappear.

## Acceptance Criteria
1. `WorkItem` enum gains a `SwitchModel(String)` variant alongside the existing `Audio(PathBuf)` and `Truncated(PathBuf)` variants
2. The worker function initialises a `WhisperEngine` from the model named in settings before entering the receive loop; the tray receives `UiMessage::SetStatus("Loading model...")` before load and `UiMessage::SetStatus("Ready")` after
3. On `WorkItem::Audio(path)` or `WorkItem::Truncated(path)`, the worker reads the WAV file as `Vec<f32>` at 16 kHz using `hound`, calls `engine.transcribe(&samples, &language, prompt)`, types the result via the existing `Typer`, and logs the elapsed time
4. On `WorkItem::SwitchModel(name)`, the worker drops the current `WhisperEngine`, emits `Loading model...` status, constructs a new engine with the requested model, and emits `Ready` — if load fails, the worker logs the error, emits `Transcription error` status, and keeps the previous engine (if any)
5. Language switching in the tray sends `WorkItem::SwitchModel(current_model_with_updated_en_suffix)` to the worker, causing a reload
6. If `engine.transcribe` returns an error, the worker logs it with the audio path and elapsed time, emits `Transcription failed` status briefly, and continues — the engine is not dropped unless the error is unrecoverable
7. The worker thread does not hold any subprocess handle, does not import the `std::process::Command` API in the transcription path
8. Dead imports (`use std::process::Command`) and dead helpers in `src/transcribe.rs` (`run_with_timeout`, `transcribe_ct2`, `transcribe_whisper_cpp`) are removed after the rewrite compiles

## Technical Approach
Introduce a small helper `load_audio_wav(path: &Path) -> anyhow::Result<Vec<f32>>` that opens the WAV with `hound::WavReader`, asserts 16 kHz mono 16-bit format (panicking if mismatched would mask audio-pipeline bugs so return `Err` instead), and collects samples converted to `f32` in range [-1.0, 1.0].

The worker loop becomes:
```rust
let mut engine = WhisperEngine::new(&model_path, model_name)?;
ui_tx.send(UiMessage::SetStatus("Ready".into())).ok();
while let Ok(item) = rx.recv() {
    match item {
        WorkItem::Audio(path) | WorkItem::Truncated(path) => {
            let samples = load_audio_wav(&path)?;
            let text = engine.transcribe(&samples, &lang, prompt.as_deref())?;
            // type, save, cleanup as before
        }
        WorkItem::SwitchModel(name) => {
            drop(engine);
            ui_tx.send(UiMessage::SetStatus("Loading model...".into())).ok();
            engine = WhisperEngine::new(&resolve_model_path(&name)?, name)?;
            ui_tx.send(UiMessage::SetStatus("Ready".into())).ok();
        }
    }
}
```

## Test Strategy
End-to-end manual smoke test covered by TASK-VTT034: record three clips with the default model, switch to `medium`, record three more, verify all six transcribe correctly with sub-second latency after initial load.

## Files
- `src/main.rs` (modify — `WorkItem` enum, worker function, add `load_audio_wav` helper)
- `src/transcribe.rs` (delete or reduce to a stub — logic moves into the worker + engine)
