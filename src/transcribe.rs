//! Thin bridge between the worker and the WhisperEngine. No subprocess, no Python.
//! v2.0.0 replaces the pre-2.0 transcribe_ct2 / transcribe_whisper_cpp subprocess
//! paths with in-process whisper-rs inference via WhisperEngine.

use crate::whisper::WhisperEngine;

/// Transcribe f32 PCM samples using the engine. Returns None on empty/error.
pub fn transcribe_samples(
    engine: &WhisperEngine,
    samples: &[f32],
    language: &str,
    prompt: &str,
) -> Option<String> {
    let prompt_opt = if prompt.is_empty() {
        None
    } else {
        Some(prompt)
    };
    match engine.transcribe(samples, language, prompt_opt) {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                crate::vtt_log!("Empty transcription result");
                None
            } else {
                Some(trimmed)
            }
        }
        Err(e) => {
            crate::vtt_log!("Transcription failed: {}", e);
            None
        }
    }
}

// load_wav() removed 2026-04-20 — was planned for --file mode (TASK-VTT023)
// which never shipped. Reintroduce from git history if --file is implemented.
