//! ParakeetEngine — NVIDIA Parakeet TDT through whisper.cpp's Parakeet API
//! (ADR-0011). Owns one parakeet_context for the lifetime of the worker, the
//! way WhisperEngine owns its WhisperContext: the model loads once and every
//! transcription reuses it.
//!
//! Parakeet is a transducer, not an autoregressive text decoder: it takes no
//! initial prompt and no language (v3 detects among 25 European languages),
//! and no window is prompted with the text of the one before — the mechanism
//! behind Whisper's loops — so its output does not go through
//! WhisperEngine's repetition guard.
//!
//! The raw bindings come from the patched whisper-rs-sys, generated from the
//! same `parakeet.h` the library is compiled from. This module is the only
//! place they are called.

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::path::Path;
use std::ptr::NonNull;
use std::sync::Once;
use std::time::Instant;
use whisper_rs_sys as sys;

/// One second of silence at 16 kHz: enough to run every graph once.
const WARM_UP_SAMPLES: usize = crate::whisper::WHISPER_INPUT_RATE as usize;
/// Recordings longer than this are decoded in chunks (TASK-VTT199). Fed an
/// 84 s dictation whole, Parakeet dropped three whole phrases, the last
/// sentence among them; the same audio in 20 s pieces lost none.
const CHUNK_SECS: f32 = 20.0;
/// How long past `CHUNK_SECS` a chunk waits for a pause before it is cut at
/// the quietest stretch instead, so no chunk runs past 30 s.
const CHUNK_PAUSE_WAIT_SECS: f32 = 10.0;

pub struct ParakeetEngine {
    ctx: NonNull<sys::parakeet_context>,
    model_name: String,
}

impl ParakeetEngine {
    /// Load a Parakeet ggml model from disk onto the first GPU ggml finds,
    /// or the CPU when there is none.
    pub fn new(model_path: &Path, model_name: impl Into<String>) -> anyhow::Result<Self> {
        let name = model_name.into();
        let path = model_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("model path is not valid UTF-8"))?;
        let c_path =
            CString::new(path).map_err(|_| anyhow::anyhow!("model path contains a NUL byte"))?;
        install_log_hook();

        let t0 = Instant::now();
        // SAFETY: `c_path` is a valid NUL-terminated string that outlives the
        // call, and the params are plain data passed by value. A NULL return
        // is a failed load, handled below.
        let raw = unsafe {
            let params = sys::parakeet_context_default_params();
            sys::parakeet_init_from_file_with_params(c_path.as_ptr(), params)
        };
        let ctx = NonNull::new(raw)
            .ok_or_else(|| anyhow::anyhow!("load model {}: Parakeet could not load it", name))?;
        let engine = Self {
            ctx,
            model_name: name,
        };
        let loaded = t0.elapsed().as_secs_f32();

        // The first call in a process builds the GPU pipelines, which takes
        // seconds while the driver's pipeline cache is cold. Pay that here,
        // behind the tray's "Loading" status, not on the first dictation.
        let warm = Instant::now();
        if let Err(e) = engine.transcribe(&[0.0; WARM_UP_SAMPLES]) {
            crate::whisper::note(&format!("Parakeet warm-up failed: {e}"));
        }

        let size_mb = std::fs::metadata(model_path)
            .map(|m| m.len() / 1_048_576)
            .unwrap_or(0);
        crate::vtt_log!(
            "Model loaded: {} ({} MB) in {:.2}s, warmed up in {:.2}s",
            engine.model_name,
            size_mb,
            loaded,
            warm.elapsed().as_secs_f32()
        );
        Ok(engine)
    }

    /// Transcribe raw 16 kHz mono f32 PCM. A recording up to `CHUNK_SECS` is
    /// one `parakeet_full` call, as it always was; a longer one is cut at
    /// pauses into chunks of about that length, each decoded on its own, and
    /// their text joined in order (TASK-VTT199).
    pub fn transcribe(&self, samples: &[f32]) -> anyhow::Result<String> {
        let chunks = chunk_ranges(samples);
        if chunks.len() == 1 {
            return self.transcribe_once(samples);
        }
        let texts: anyhow::Result<Vec<String>> = chunks
            .iter()
            .map(|&(start, end)| self.transcribe_once(&samples[start..end]))
            .collect();
        crate::whisper::note(&format!(
            "Parakeet: {:.1}s recording decoded as {} chunks cut at pauses",
            samples.len() as f32 / crate::whisper::WHISPER_INPUT_RATE as f32,
            chunks.len()
        ));
        Ok(crate::whisper::join_trimmed(&texts?))
    }

    /// One `parakeet_full` call over `samples`. Each call starts from a
    /// cleared decoder state, so no text or LSTM state crosses from one
    /// dictation, or one chunk, to the next.
    fn transcribe_once(&self, samples: &[f32]) -> anyhow::Result<String> {
        if samples.is_empty() {
            return Ok(String::new());
        }
        let n_samples = c_int::try_from(samples.len())
            .map_err(|_| anyhow::anyhow!("{} samples is too long for one call", samples.len()))?;
        let ctx = self.ctx.as_ptr();

        // SAFETY: `ctx` is a live context owned by `self` and used from one
        // thread at a time (`&self` here, the worker owns the engine);
        // `samples` is valid for `n_samples` reads for the whole call.
        let rc = unsafe {
            let mut params = sys::parakeet_full_default_params(
                sys::parakeet_sampling_strategy_PARAKEET_SAMPLING_GREEDY,
            );
            params.n_threads = 4;
            params.no_context = true;
            sys::parakeet_full(ctx, params, samples.as_ptr(), n_samples)
        };
        if rc != 0 {
            anyhow::bail!("Parakeet transcription failed (code {rc})");
        }

        // SAFETY: segment indices stay below the count the library reports,
        // and each returned pointer is a NUL-terminated string owned by the
        // context's state, valid until the next `parakeet_full`; it is copied
        // before this function returns.
        let parts: Vec<String> = unsafe {
            let n_segments = sys::parakeet_full_n_segments(ctx);
            (0..n_segments)
                .filter_map(|i| {
                    let text = sys::parakeet_full_get_segment_text(ctx, i);
                    (!text.is_null()).then(|| CStr::from_ptr(text).to_string_lossy().into_owned())
                })
                .collect()
        };
        Ok(crate::whisper::join_trimmed(&parts))
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }
}

impl Drop for ParakeetEngine {
    fn drop(&mut self) {
        // SAFETY: the context came from parakeet_init_* and is freed exactly
        // once, here.
        unsafe { sys::parakeet_free(self.ctx.as_ptr()) };
    }
}

/// Pure: where a recording at 16 kHz is cut for Parakeet. Up to `CHUNK_SECS`
/// it is one range, the whole recording, untouched; longer, it is cut at the
/// first pause after every `CHUNK_SECS`, by the same rules that cut a held
/// key's capture, into contiguous ranges covering every sample.
fn chunk_ranges(samples: &[f32]) -> Vec<(usize, usize)> {
    let rate = crate::whisper::WHISPER_INPUT_RATE;
    if samples.len() as f32 <= CHUNK_SECS * rate as f32 {
        return vec![(0, samples.len())];
    }
    let policy = crate::segment::Policy::with_wait(rate, CHUNK_SECS, CHUNK_PAUSE_WAIT_SECS);
    crate::segment::split(samples, rate, policy)
}

/// Route Parakeet's own log lines: errors to the log (or stderr in `--file`
/// mode), everything else dropped, matching what whisper-rs's hooks do for
/// whisper.cpp. Parakeet would otherwise print every load and timing line to
/// stderr.
fn install_log_hook() {
    static ONCE: Once = Once::new();
    // SAFETY: registers a callback with static lifetime and no user data.
    ONCE.call_once(|| unsafe { sys::parakeet_log_set(Some(parakeet_log), std::ptr::null_mut()) });
}

unsafe extern "C" fn parakeet_log(
    level: sys::ggml_log_level,
    text: *const c_char,
    _user_data: *mut c_void,
) {
    if level != sys::ggml_log_level_GGML_LOG_LEVEL_ERROR || text.is_null() {
        return;
    }
    // SAFETY: non-null, NUL-terminated per the ggml log contract, and only
    // read for the duration of this call.
    let line = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    // Never unwind into C.
    let _ =
        std::panic::catch_unwind(|| crate::whisper::note(&format!("Parakeet: {}", line.trim())));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_err_for_missing_model_file() {
        // A missing file must fail cleanly, not panic, so the worker can turn
        // it into a tray "model failed" status like it does for Whisper.
        let missing = Path::new("this-parakeet-model-does-not-exist-zzz.bin");
        assert!(ParakeetEngine::new(missing, "ghost").is_err());
    }

    use crate::segment::test_support::{dictation, secs};

    #[test]
    fn a_recording_up_to_the_chunk_length_is_decoded_whole() {
        // Today's path, exactly: one range, the whole buffer.
        for len in [0, 1, secs(5.0), secs(CHUNK_SECS)] {
            assert_eq!(chunk_ranges(&vec![0.1; len]), vec![(0, len)]);
        }
    }

    #[test]
    fn a_long_recording_is_cut_into_chunks_covering_every_sample() {
        let (audio, _) = dictation(84.0, 11);
        let ranges = chunk_ranges(&audio);
        assert!(ranges.len() >= 3, "{} chunks in 84 s", ranges.len());
        let mut at = 0;
        for &(start, end) in &ranges {
            assert_eq!(start, at, "chunks must be contiguous");
            assert!(end - start <= secs(CHUNK_SECS + CHUNK_PAUSE_WAIT_SECS));
            at = end;
        }
        assert_eq!(at, audio.len(), "chunks must cover the whole recording");
    }

    #[test]
    fn no_chunk_boundary_splits_a_word() {
        for seed in 1..=30 {
            let (audio, words) = dictation(120.0, seed);
            let ranges = chunk_ranges(&audio);
            for &(_, cut) in &ranges[..ranges.len() - 1] {
                if let Some(&(s, e)) = words.iter().find(|&&(s, e)| s < cut && cut < e) {
                    panic!("seed {seed}: chunk cut at {cut} splits the word {s}..{e}");
                }
            }
        }
    }
}
