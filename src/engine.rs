//! The speech engines behind one contract (ADR-0011). The worker and `--file`
//! load an `Engine` from a catalogue entry and call `transcribe`; which
//! engine runs is a property of the model chosen, never a separate setting.

use crate::models::{self, EngineKind, ModelInfo};
use crate::parakeet::ParakeetEngine;
use crate::whisper::WhisperEngine;
use std::path::Path;

pub enum Engine {
    Whisper(WhisperEngine),
    Parakeet(ParakeetEngine),
}

impl Engine {
    /// Load the model at `path` with the engine its catalogue entry names.
    /// Whisper also gets the Silero VAD model for long recordings, downloaded
    /// on first use; without it Whisper still loads and the repetition guard
    /// stays the only defence against loops.
    pub fn load(info: &ModelInfo, path: &Path) -> anyhow::Result<Self> {
        Ok(match info.engine {
            EngineKind::Whisper => {
                let mut engine = WhisperEngine::new(path, info.name)?;
                let vad = models::ensure(&models::VAD_MODEL, |_, _| {})
                    .and_then(|vad| engine.set_vad_model(&vad));
                if let Err(e) = vad {
                    crate::whisper::note(&format!(
                        "VAD unavailable ({e}); long recordings decode without it"
                    ));
                }
                Self::Whisper(engine)
            }
            EngineKind::Parakeet => Self::Parakeet(ParakeetEngine::new(path, info.name)?),
        })
    }

    /// Transcribe 16 kHz mono f32 PCM. `language` and `prompt` steer Whisper
    /// only: Parakeet detects the language itself and takes no prompt, so
    /// for it the corrections dictionary is the only vocabulary lever.
    pub fn transcribe(
        &self,
        samples: &[f32],
        language: &str,
        prompt: Option<&str>,
    ) -> anyhow::Result<String> {
        match self {
            Self::Whisper(e) => e.transcribe(samples, language, prompt),
            Self::Parakeet(e) => e.transcribe(samples),
        }
    }

    pub fn model_name(&self) -> &str {
        match self {
            Self::Whisper(e) => e.model_name(),
            Self::Parakeet(e) => e.model_name(),
        }
    }
}

/// The whisper.cpp release this binary was built from, as the library itself
/// reports it. Parakeet ships inside the same release.
pub fn version() -> &'static str {
    whisper_rs::get_whisper_version()
}
