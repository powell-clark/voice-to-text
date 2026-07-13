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
        crate::vtt_log!("Model loaded: {} ({} MB) in {:.2}s", name, size_mb, elapsed);

        Ok(Self {
            ctx,
            model_name: name,
        })
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

/// Decode a 16 kHz WAV file into the mono f32 PCM samples whisper expects
/// (`--file` batch mode, TASK-VTT023). Stereo is down-mixed by averaging
/// channels; 16-bit (and wider) integer and 32-bit float sample formats are
/// supported. A non-16 kHz file is rejected with an actionable error rather
/// than silently mis-transcribed — resampling is out of scope for this pass.
pub fn decode_wav_to_samples(path: &Path) -> anyhow::Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| anyhow::anyhow!("open {}: {}", path.display(), e))?;
    let spec = reader.spec();
    if spec.sample_rate != 16_000 {
        anyhow::bail!(
            "{} is {} Hz — --file needs 16 kHz mono audio. Resample first, e.g.: \
             `ffmpeg -i in.wav -ar 16000 -ac 1 out.wav`",
            path.display(),
            spec.sample_rate
        );
    }

    // Read every interleaved sample as f32 in roughly [-1, 1].
    let interleaved: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            // Normalise by the format's full-scale so 16/24/32-bit all land in
            // the same range. `samples::<i32>()` sign-extends narrower depths.
            let full_scale = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / full_scale))
                .collect::<Result<_, _>>()?
        }
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>()?,
    };

    let channels = spec.channels.max(1) as usize;
    if channels == 1 {
        return Ok(interleaved);
    }
    Ok(interleaved
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Write `samples` (interleaved) as a 16-bit PCM WAV at `rate`/`channels`
    /// into a fresh temp file and return the temp dir (keep it alive) + path.
    fn write_test_wav(samples: &[f32], rate: u32, channels: u16) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("clip.wav");
        let spec = hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&path, spec).expect("create wav");
        for &s in samples {
            w.write_sample((s * 32767.0) as i16).expect("write sample");
        }
        w.finalize().expect("finalize wav");
        (dir, path)
    }

    #[test]
    fn decode_wav_reads_mono_16k_into_normalised_samples() {
        let input: Vec<f32> = (0..64).map(|i| i as f32 / 64.0 - 0.5).collect();
        let (_dir, path) = write_test_wav(&input, 16_000, 1);
        let out = decode_wav_to_samples(&path).expect("decode mono");
        assert_eq!(out.len(), input.len(), "mono sample count preserved");
        for (a, b) in input.iter().zip(out.iter()) {
            assert!((a - b).abs() < 0.001, "sample within i16 quantisation");
        }
    }

    #[test]
    fn decode_wav_downmixes_stereo_to_mono() {
        // Frames of (L, R): averaging gives the mono value. 4 frames -> 4 samples.
        let interleaved = [0.5, -0.5, 0.2, 0.2, 1.0, 0.0, -0.4, -0.4];
        let (_dir, path) = write_test_wav(&interleaved, 16_000, 2);
        let out = decode_wav_to_samples(&path).expect("decode stereo");
        assert_eq!(out.len(), 4, "stereo down-mixed to one sample per frame");
        let expected = [0.0, 0.2, 0.5, -0.4];
        for (a, b) in expected.iter().zip(out.iter()) {
            assert!((a - b).abs() < 0.001, "channel average: got {b}, want {a}");
        }
    }

    #[test]
    fn decode_wav_rejects_non_16khz() {
        let (_dir, path) = write_test_wav(&[0.1, 0.2, 0.3], 44_100, 1);
        let err = decode_wav_to_samples(&path).expect_err("44.1 kHz must be rejected");
        assert!(
            err.to_string().contains("16 kHz"),
            "error should name the required rate: {err}"
        );
    }

    #[test]
    fn new_returns_err_for_missing_model_file() {
        // Loading a non-existent model path must fail cleanly with an error,
        // not panic — the worker turns this into a tray "model failed" status.
        let missing = Path::new("this-model-does-not-exist-zzz.bin");
        let result = WhisperEngine::new(missing, "ghost");
        assert!(
            result.is_err(),
            "missing model should return Err, not panic"
        );
    }

    // ── End-to-end transcription (opt-in) ─────────────────────────────────────
    // Proves the real audio->text path on this machine: a committed WAV of
    // synthesized speech (16 kHz mono, see tests/fixtures/) is decoded and run
    // through whisper-rs / whisper.cpp, and the transcript must contain the
    // spoken digits. Ignored by default because it downloads a ~142 MB model
    // (base.en) on first run and takes a few seconds of CPU inference. Run with:
    //
    //     cargo test --release -- --ignored e2e_transcribes
    //
    // The fixture is generated by scripts/gen-test-fixture-windows.ps1 (SAPI).
    const E2E_MODEL_URL: &str =
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";
    const E2E_MODEL_FILE: &str = "ggml-base.en.bin";

    fn e2e_model_path() -> PathBuf {
        let dir = dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("voice-to-text/models");
        std::fs::create_dir_all(&dir).ok();
        dir.join(E2E_MODEL_FILE)
    }

    fn ensure_e2e_model() -> PathBuf {
        let path = e2e_model_path();
        if path.exists() {
            return path;
        }
        let tmp = path.with_extension("bin.tmp");
        let bytes = reqwest::blocking::Client::builder()
            .timeout(None)
            .build()
            .expect("build http client")
            .get(E2E_MODEL_URL)
            .send()
            .expect("download base.en")
            .error_for_status()
            .expect("base.en http status")
            .bytes()
            .expect("read base.en body");
        std::fs::write(&tmp, &bytes).expect("write model tmp");
        std::fs::rename(&tmp, &path).expect("rename model into place");
        path
    }

    /// Decode a 16 kHz mono 16-bit PCM WAV into the f32 samples whisper expects.
    fn load_wav_f32(path: &Path) -> Vec<f32> {
        let mut reader = hound::WavReader::open(path).expect("open fixture wav");
        let spec = reader.spec();
        assert_eq!(spec.channels, 1, "fixture must be mono");
        assert_eq!(spec.sample_rate, 16_000, "fixture must be 16 kHz");
        assert_eq!(spec.bits_per_sample, 16, "fixture must be 16-bit");
        reader
            .samples::<i16>()
            .map(|s| s.expect("read sample") as f32 / 32_768.0)
            .collect()
    }

    #[test]
    #[ignore = "downloads ~142 MB base.en model and runs CPU inference"]
    fn e2e_transcribes_spoken_digits_from_fixture() {
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/testing-one-two-three.wav");
        assert!(
            fixture.exists(),
            "missing fixture {} — run scripts/gen-test-fixture-windows.ps1",
            fixture.display()
        );

        let samples = load_wav_f32(&fixture);
        assert!(!samples.is_empty(), "fixture decoded to zero samples");

        let model = ensure_e2e_model();
        let engine = WhisperEngine::new(&model, "base.en").expect("load base.en");
        let text = engine
            .transcribe(&samples, "en", None)
            .expect("transcription should succeed");

        // Normalise: lowercase, keep alphanumerics, split to a word set.
        let normalised: String = text
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect();
        let words: std::collections::HashSet<&str> = normalised.split_whitespace().collect();

        // Content words whisper renders literally and reliably from clean speech.
        for expected in ["testing", "quick", "brown", "fox", "lazy", "dog"] {
            assert!(
                words.contains(expected),
                "transcript {:?} missing expected word {:?}",
                text,
                expected
            );
        }

        // The spoken digits "one two three four" — whisper normalises these to
        // "1234" (or "1 2 3 4"), so accept either the numeric or spelled forms
        // rather than asserting one rendering.
        let digits_ok = normalised.contains("1234")
            || ["1", "2", "3", "4"].iter().all(|d| words.contains(d))
            || ["one", "two", "three", "four"]
                .iter()
                .all(|w| words.contains(w));
        assert!(
            digits_ok,
            "transcript {:?} missing the spoken digits in any recognised form",
            text
        );
    }
}
