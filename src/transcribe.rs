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
            let result = finalize(text);
            if result.is_none() {
                crate::vtt_log!("Empty transcription result");
            }
            result
        }
        Err(e) => {
            crate::vtt_log!("Transcription failed: {}", e);
            None
        }
    }
}

/// Trim surrounding whitespace and collapse an all-whitespace (or empty) result
/// to `None`. Pure so it can be unit-tested without a loaded model — whisper
/// occasionally returns blank or whitespace-only segments for silence, and the
/// caller must not type those.
fn finalize(text: String) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

// load_wav() removed 2026-04-20 — was planned for --file mode (TASK-VTT023)
// which never shipped. Reintroduce from git history if --file is implemented.

#[cfg(test)]
mod tests {
    use super::finalize;

    #[test]
    fn finalize_trims_surrounding_whitespace() {
        assert_eq!(
            finalize("  hello world  ".into()),
            Some("hello world".into())
        );
        assert_eq!(finalize("\n\tindented\n".into()), Some("indented".into()));
    }

    #[test]
    fn finalize_collapses_empty_and_whitespace_to_none() {
        assert_eq!(finalize(String::new()), None);
        assert_eq!(finalize("   ".into()), None);
        assert_eq!(finalize("\n\t  \r\n".into()), None);
    }

    #[test]
    fn finalize_preserves_internal_whitespace_and_unicode() {
        assert_eq!(
            finalize("  £100 — naïve  café  ".into()),
            Some("£100 — naïve  café".into()),
            "internal spacing and non-ASCII must survive trimming"
        );
    }
}
