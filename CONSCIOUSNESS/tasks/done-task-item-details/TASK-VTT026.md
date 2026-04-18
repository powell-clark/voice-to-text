# TASK-VTT026: WhisperEngine wrapper in src/whisper.rs

## Context
The Rust code needs a single owned abstraction over `whisper_rs::WhisperContext` and `WhisperState` so the transcription worker can load a model once, reuse it across transcriptions, and swap it out when the user selects a different model. This task creates `src/whisper.rs` exposing a clean API that hides the raw whisper-rs surface from the rest of the code.

## Acceptance Criteria
1. `src/whisper.rs` defines `pub struct WhisperEngine { ctx: WhisperContext, model_name: String }` — the state is created per-transcription (state is single-use), but the context is retained
2. `WhisperEngine::new(model_path: &Path, model_name: String) -> anyhow::Result<Self>` constructs the context with `WhisperContextParameters::default()` and logs the model name, file size, and load duration to `vtt_log`
3. `WhisperEngine::transcribe(&self, samples: &[f32], language: &str, prompt: Option<&str>) -> anyhow::Result<String>` creates a fresh `WhisperState`, configures `FullParams` with language, optional initial prompt, `beam_size = 1` (greedy), `no_timestamps = true`, and `word_thold = 0.01`, runs `state.full(params, samples)`, collects all segment texts with trimmed whitespace, joins with spaces, and returns the result
4. `WhisperEngine::transcribe` returns `Err` if inference panics or returns a whisper error; the caller decides whether to recreate the engine (user-selected model change) or retry (transient failure)
5. `WhisperEngine::model_name() -> &str` exposes the currently loaded model label for the tray status display
6. The struct does not implement `Clone` (contexts are heavy); users wrap it in `Arc` if shared ownership is needed
7. The file has a module-level doc comment explaining the architectural role: "Owns the loaded Whisper model for the lifetime of the transcription worker thread"

## Technical Approach
```rust
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperEngine {
    ctx: WhisperContext,
    model_name: String,
}

impl WhisperEngine {
    pub fn new(model_path: &Path, model_name: String) -> anyhow::Result<Self> {
        let t0 = std::time::Instant::now();
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or_else(|| anyhow::anyhow!("non-utf8 path"))?,
            WhisperContextParameters::default(),
        ).map_err(|e| anyhow::anyhow!("load model {}: {:?}", model_name, e))?;
        crate::vtt_log!("Model loaded: {} in {:.2}s", model_name, t0.elapsed().as_secs_f32());
        Ok(Self { ctx, model_name })
    }

    pub fn transcribe(&self, samples: &[f32], language: &str, prompt: Option<&str>) -> anyhow::Result<String> {
        let mut state = self.ctx.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(language));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        if let Some(p) = prompt { params.set_initial_prompt(p); }
        state.full(params, samples)?;
        let n = state.full_n_segments()?;
        let mut text = String::new();
        for i in 0..n {
            let seg = state.full_get_segment_text(i)?;
            if !text.is_empty() { text.push(' '); }
            text.push_str(seg.trim());
        }
        Ok(text)
    }

    pub fn model_name(&self) -> &str { &self.model_name }
}
```

## Test Strategy
Unit test (Rust `#[cfg(test)]`): construct a `WhisperEngine` with a bundled `ggml-tiny.en.bin` (committed as test fixture), feed a synthetic 1-second f32 sine-wave buffer, assert transcription completes without error and returns an empty or non-empty string. Integration test deferred to TASK-VTT034 (end-to-end smoke test).

## Files
- `src/whisper.rs` (create)
- `src/main.rs` (modify — add `mod whisper;`)
