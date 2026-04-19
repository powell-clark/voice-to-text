//! Thin bridge between the worker and the WhisperEngine. No subprocess, no Python.
//! v2.0.0 replaces the pre-2.0 transcribe_ct2 / transcribe_whisper_cpp subprocess
//! paths with in-process whisper-rs inference via WhisperEngine.

use std::path::Path;

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

/// Read a 16kHz mono WAV file into a Vec<f32>. Used when the worker receives a
/// path-based WorkItem (from the debug recordings archive or a --file mode).
pub fn load_wav(path: &Path) -> anyhow::Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    if spec.sample_rate != 16000 || spec.channels != 1 {
        anyhow::bail!(
            "WAV must be 16kHz mono, got {}Hz {}ch",
            spec.sample_rate,
            spec.channels
        );
    }
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .filter_map(Result::ok)
            .map(|s| s as f32 / 32768.0)
            .collect(),
        hound::SampleFormat::Float => reader.samples::<f32>().filter_map(Result::ok).collect(),
    };
    Ok(samples)
}
