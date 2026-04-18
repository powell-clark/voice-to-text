//! WhisperEngine — owns a WhisperContext for the lifetime of the transcription worker
//! thread. The model loads once, stays resident, and all transcriptions reuse it.
//!
//! Replaces the pre-v2.0 architecture which spawned `python3 transcribe.py` per press
//! and paid the full model-load cost every transcription.

use std::path::Path;
use std::time::Instant;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperEngine {
    ctx: WhisperContext,
    model_name: String,
}

impl WhisperEngine {
    /// Load a GGML model from disk into memory. Takes seconds for first-load;
    /// the returned engine is reused for every subsequent transcription.
    pub fn new(model_path: &Path, model_name: impl Into<String>) -> anyhow::Result<Self> {
        let name = model_name.into();
        let path_str = model_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("model path is not valid UTF-8"))?;

        let t0 = Instant::now();
        let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
            .map_err(|e| anyhow::anyhow!("load model {}: {:?}", name, e))?;
        let elapsed = t0.elapsed().as_secs_f32();

        let size_mb = std::fs::metadata(model_path)
            .map(|m| m.len() / 1_048_576)
            .unwrap_or(0);
        crate::vtt_log!(
            "Model loaded: {} ({} MB) in {:.2}s",
            name,
            size_mb,
            elapsed
        );

        Ok(Self { ctx, model_name: name })
    }

    /// Transcribe raw 16kHz mono f32 PCM samples. Creates a fresh WhisperState each
    /// call (whisper-rs states are single-use) but reuses the owned WhisperContext.
    pub fn transcribe(
        &self,
        samples: &[f32],
        language: &str,
        prompt: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut state = self.ctx.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_language(Some(language));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_single_segment(false);
        params.set_n_threads(4);

        if let Some(p) = prompt {
            if !p.is_empty() {
                params.set_initial_prompt(p);
            }
        }

        state.full(params, samples)?;

        let mut text = String::new();
        for segment in state.as_iter() {
            let seg_text = segment.to_string();
            let trimmed = seg_text.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(trimmed);
        }
        Ok(text)
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }
}
