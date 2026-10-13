//! WhisperEngine — owns a WhisperContext for the lifetime of the transcription worker
//! thread. The model loads once, stays resident, and all transcriptions reuse it.
//!
//! Replaces the pre-v2.0 architecture which spawned `python3 transcribe.py` per press
//! and paid the full model-load cost every transcription.

/// The only input rate every supported Whisper model is trained at.
/// Defined here because this module is what rejects anything else.
pub const WHISPER_INPUT_RATE: u32 = 16_000;

/// A sentence, or a block of up to `MAX_LOOP_PERIOD` sentences, repeated
/// this many times back to back is a decoder loop (TASK-VTT179).
const LOOP_REPEATS: usize = 3;
/// whisper.cpp loops a whole segment, and one segment can hold more than
/// one sentence, so a loop's repeating unit may be a short block.
const MAX_LOOP_PERIOD: usize = 3;
/// Chunk length when the repetition guard re-decodes: under one 30 s
/// window, so a chunk has no earlier window whose text it could inherit.
const MAX_CHUNK_SECS: usize = 28;
/// How far either side of an even split to look for a pause to cut at.
const CUT_SLACK_SECS: usize = 4;
/// Length of the quietest stretch a cut is centred on.
const QUIET_WINDOW_MS: usize = 300;
/// Piece length when a chunk loops on its own and is decoded once more.
const PIECE_SECS: usize = 10;
const PIECE_SLACK_SECS: usize = 3;

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

    /// Transcribe raw 16kHz mono f32 PCM samples in one `full` call. A
    /// transcript that loops goes to the repetition guard (TASK-VTT179).
    pub fn transcribe(
        &self,
        samples: &[f32],
        language: &str,
        prompt: Option<&str>,
    ) -> anyhow::Result<String> {
        let text = self.decode(samples, language, prompt, true)?;
        let repeats = longest_loop(&text);
        if repeats < LOOP_REPEATS {
            return Ok(text);
        }
        Ok(self.repetition_guard(samples, language, prompt, text, repeats))
    }

    /// Repetition guard (TASK-VTT179). Within one `full` call whisper.cpp
    /// walks the buffer in 30 s windows and prompts each window with the
    /// text decoded for the windows before it, so one bad window can repeat
    /// itself to the end: a 178 s dictation lost its second half that way.
    /// The guard re-decodes the audio as pause-aligned chunks of at most
    /// `MAX_CHUNK_SECS`, each its own call carrying only the initial prompt,
    /// so no decoded text crosses from one chunk to the next. A chunk that
    /// loops on its own is decoded again in shorter pieces with no context
    /// at all, and whatever still repeats is collapsed, so the operator never
    /// sees the loop. Logs one line of counts, never text.
    ///
    /// Decoding without carried text is not the default because it drops
    /// words context would have recovered: up to 12% on long dictations
    /// that never looped.
    fn repetition_guard(
        &self,
        samples: &[f32],
        language: &str,
        prompt: Option<&str>,
        looped: String,
        repeats: usize,
    ) -> String {
        let rate = WHISPER_INPUT_RATE as usize;
        let secs = samples.len() as f32 / rate as f32;
        let ranges = pause_aligned_ranges(
            samples,
            MAX_CHUNK_SECS * rate,
            CUT_SLACK_SECS * rate,
            QUIET_WINDOW_MS * rate / 1000,
        );
        let mut redone = 0;
        let redecoded: anyhow::Result<Vec<String>> = ranges
            .iter()
            .map(|&(start, end)| {
                let chunk = &samples[start..end];
                // A buffer that is already one chunk would decode the same
                // again; go straight to pieces.
                let text = if ranges.len() == 1 {
                    looped.clone()
                } else {
                    self.decode(chunk, language, prompt, true)?
                };
                if longest_loop(&text) < LOOP_REPEATS {
                    return Ok(text);
                }
                redone += 1;
                self.decode_pieces(chunk, language)
            })
            .collect();
        let (text, outcome) = match redecoded {
            Ok(parts) => (
                join_trimmed(&parts),
                format!(
                    "re-decoded as {} pause-aligned chunks without carried text, \
                     {redone} decoded again in pieces",
                    ranges.len()
                ),
            ),
            Err(e) => (looped, format!("re-decode failed ({e}), kept first decode")),
        };
        let after = longest_loop(&text);
        let (text, removed) = collapse_loops(&text, LOOP_REPEATS);
        note(&format!(
            "Repetition guard: {secs:.1}s decode looped {repeats}x; {outcome}; \
             longest loop now {after}x, {removed} repeated sentences collapsed"
        ));
        text
    }

    /// Decode `chunk` as pieces of at most `PIECE_SECS` with no context at
    /// all — neither earlier decoded text nor the initial prompt, since a
    /// chunk that loops inside one window may be looping on either.
    fn decode_pieces(&self, chunk: &[f32], language: &str) -> anyhow::Result<String> {
        let rate = WHISPER_INPUT_RATE as usize;
        let pieces: anyhow::Result<Vec<String>> = pause_aligned_ranges(
            chunk,
            PIECE_SECS * rate,
            PIECE_SLACK_SECS * rate,
            QUIET_WINDOW_MS * rate / 1000,
        )
        .iter()
        .map(|&(s, e)| self.decode(&chunk[s..e], language, None, false))
        .collect();
        Ok(join_trimmed(&pieces?))
    }

    /// One `full` call over `samples`. Creates a fresh WhisperState each call
    /// (whisper-rs states are single-use) but reuses the owned WhisperContext.
    /// `carry_context` false sets n_max_text_ctx to 0, which whisper.cpp
    /// reads as "prompt no window with anything": no decoded text from an
    /// earlier window and no initial prompt either.
    fn decode(
        &self,
        samples: &[f32],
        language: &str,
        prompt: Option<&str>,
        carry_context: bool,
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
        if !carry_context {
            params.set_n_max_text_ctx(0);
        }

        if let Some(p) = prompt {
            if !p.is_empty() {
                params.set_initial_prompt(p);
            }
        }

        state.full(params, samples)?;

        let segments: Vec<String> = state.as_iter().map(|s| s.to_string()).collect();
        Ok(join_trimmed(&segments))
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }
}

/// Join trimmed, non-empty pieces of text with single spaces.
fn join_trimmed(parts: &[String]) -> String {
    parts
        .iter()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Diagnostics go to the log file, or to stderr when no log file is open
/// (`--file` mode leaves the logger uninitialised to keep stdout clean).
fn note(msg: &str) {
    if crate::logging::get_dir().as_os_str().is_empty() {
        eprintln!("{msg}");
    } else {
        crate::vtt_log!("{msg}");
    }
}

/// Pure: split `samples` into contiguous ranges of at most `max_len` samples
/// covering the whole buffer. Each cut lands at the centre of the quietest
/// `quiet`-sample stretch within `slack` of an even split of what remains,
/// so chunks break in pauses rather than mid-word. A buffer that already
/// fits (empty included) is returned as one range.
fn pause_aligned_ranges(
    samples: &[f32],
    max_len: usize,
    slack: usize,
    quiet: usize,
) -> Vec<(usize, usize)> {
    let total = samples.len();
    let max_len = max_len.max(1);
    if total <= max_len {
        return vec![(0, total)];
    }
    // An even split of what remains is at least max_len/2 ahead, so capping
    // the slack at max_len/4 moves every cut at least max_len/4 forward.
    let slack = slack.min(max_len / 4);
    let half = (quiet / 2).max(1);
    let hop = (quiet / 30).max(1);

    // Prefix sums of squared samples: the energy of any stretch in O(1).
    let mut energy = Vec::with_capacity(total + 1);
    let mut acc = 0.0f64;
    energy.push(acc);
    for &s in samples {
        acc += f64::from(s) * f64::from(s);
        energy.push(acc);
    }

    let mut ranges = Vec::new();
    let mut start = 0;
    let max_steps = total / (max_len / 4).max(1) + 2;
    for _ in 0..max_steps {
        let remaining = total - start;
        if remaining <= max_len {
            break;
        }
        let target = start + remaining / remaining.div_ceil(max_len);
        let lo = target.saturating_sub(slack).max(start + 1);
        let hi = (target + slack).min(start + max_len);
        let cut = quietest_point(&energy, lo, hi, half, hop);
        ranges.push((start, cut));
        start = cut;
    }
    // Always the tail, so no audio is ever dropped.
    ranges.push((start, total));
    ranges
}

/// The point in `lo..=hi` (stepping by `hop`) whose `±half` neighbourhood
/// has the least energy; the earliest wins a tie.
fn quietest_point(energy: &[f64], lo: usize, hi: usize, half: usize, hop: usize) -> usize {
    let last = energy.len().saturating_sub(1);
    let mut best = (lo, f64::INFINITY);
    for at in (lo..=hi).step_by(hop) {
        let e = energy[(at + half).min(last)] - energy[at.saturating_sub(half).min(last)];
        if e < best.1 {
            best = (at, e);
        }
    }
    best.0
}

/// Pure: split `text` into sentences, each ending at `.`, `!`, `?` or `…`
/// followed by whitespace or the end, paired with a comparison key — its
/// words lowercased, punctuation dropped — so "Okay." and "okay!" match.
fn sentences(text: &str) -> Vec<(&str, String)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let ends = matches!(c, '.' | '!' | '?' | '…')
            && chars.peek().is_none_or(|&(_, next)| next.is_whitespace());
        if ends {
            let end = i + c.len_utf8();
            push_sentence(&mut out, &text[start..end]);
            start = end;
        }
    }
    push_sentence(&mut out, &text[start..]);
    out
}

fn push_sentence<'a>(out: &mut Vec<(&'a str, String)>, raw: &'a str) {
    let raw = raw.trim();
    if raw.is_empty() {
        return;
    }
    let key = raw
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ");
    out.push((raw, key));
}

/// The repeating block that starts at sentence `at`: `(period, repeats)` —
/// how many sentences the block holds (up to `MAX_LOOP_PERIOD`) and how many
/// times it occurs back to back. The block with the most repeats wins, the
/// shorter block on a tie. A punctuation-only sentence (empty key) is never
/// part of a repeating block.
fn loop_at(sents: &[(&str, String)], at: usize) -> (usize, usize) {
    let mut best = (1, 1);
    for period in 1..=MAX_LOOP_PERIOD {
        let Some(block) = sents.get(at..at + period) else {
            break;
        };
        if block.iter().any(|s| s.1.is_empty()) {
            continue;
        }
        let repeats = sents[at..]
            .chunks_exact(period)
            .take_while(|next| next.iter().zip(block).all(|(a, b)| a.1 == b.1))
            .count();
        if repeats > best.1 {
            best = (period, repeats);
        }
    }
    best
}

/// Pure: the most times one sentence, or one block of up to
/// `MAX_LOOP_PERIOD` sentences, repeats back to back in `text`.
fn longest_loop(text: &str) -> usize {
    let sents = sentences(text);
    (0..sents.len())
        .map(|at| loop_at(&sents, at).1)
        .max()
        .unwrap_or(0)
}

/// Pure: collapse every block repeated `min_repeats` or more times back to
/// back to its first occurrence; fewer repeats ("Thanks. Thanks.") stay.
/// Returns the text, byte for byte unchanged when nothing collapsed, and how
/// many repeated sentences were removed.
fn collapse_loops(text: &str, min_repeats: usize) -> (String, usize) {
    let sents = sentences(text);
    let mut kept: Vec<&str> = Vec::with_capacity(sents.len());
    let mut removed = 0;
    let mut at = 0;
    // Bounded: `at` moves forward by at least one sentence every pass.
    while at < sents.len() {
        let (period, repeats) = loop_at(&sents, at);
        if repeats >= min_repeats {
            kept.extend(sents[at..at + period].iter().map(|s| s.0));
            removed += period * (repeats - 1);
            at += period * repeats;
        } else {
            kept.push(sents[at].0);
            at += 1;
        }
    }
    if removed == 0 {
        return (text.to_string(), 0);
    }
    (kept.join(" "), removed)
}

/// Down-mix interleaved multi-channel f32 samples to mono by channel
/// averaging. Shared by every decode path (WAV via hound, everything else
/// via symphonia) so there is exactly one down-mix formula to get right.
fn downmix_to_mono(interleaved: &[f32], channels: usize) -> Vec<f32> {
    let channels = channels.max(1);
    if channels == 1 {
        return interleaved.to_vec();
    }
    interleaved
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Decode a WAV file into mono f32 PCM samples at its native sample rate —
/// no rate restriction, no resampling. Stereo is down-mixed by averaging
/// channels; 16-bit (and wider) integer and 32-bit float sample formats are
/// supported. Returns `(samples, native_sample_rate)`; the caller resamples
/// to 16 kHz if needed (`decode_audio_to_samples`).
fn decode_wav_native(path: &Path) -> anyhow::Result<(Vec<f32>, u32)> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| anyhow::anyhow!("open {}: {}", path.display(), e))?;
    let spec = reader.spec();

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

    let mono = downmix_to_mono(&interleaved, spec.channels as usize);
    Ok((mono, spec.sample_rate))
}

/// Decode a 16 kHz WAV file into the mono f32 PCM samples whisper expects
/// (`--file` batch mode, TASK-VTT023). A non-16 kHz file is rejected with an
/// actionable error — this function's contract is unchanged from TASK-VTT023
/// (still used by `RetranscribeLast`, which only ever sees archives this
/// app itself already captured at 16 kHz). `--file`'s own multi-rate/
/// multi-format support is `decode_audio_to_samples` (TASK-VTT130), which
/// calls `decode_wav_native` directly rather than rejecting non-16 kHz WAV.
pub fn decode_wav_to_samples(path: &Path) -> anyhow::Result<Vec<f32>> {
    let (mono, rate) = decode_wav_native(path)?;
    if rate != WHISPER_INPUT_RATE {
        anyhow::bail!(
            "{} is {} Hz — --file needs 16 kHz mono audio. Resample first, e.g.: \
             `ffmpeg -i in.wav -ar 16000 -ac 1 out.wav`",
            path.display(),
            rate
        );
    }
    Ok(mono)
}

/// Decode `.mp3`, `.m4a`/`.aac`, or `.flac` into mono f32 PCM at native
/// sample rate via `symphonia` (ADR-0006). Returns `(samples,
/// native_sample_rate)`, matching `decode_wav_native`'s contract so both
/// feed the same resample step in `decode_audio_to_samples`.
fn decode_via_symphonia(path: &Path) -> anyhow::Result<(Vec<f32>, u32)> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::errors::Error as SymErr;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file =
        std::fs::File::open(path).map_err(|e| anyhow::anyhow!("open {}: {}", path.display(), e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "{}: unrecognised or unsupported audio format ({e})",
                path.display()
            )
        })?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow::anyhow!("{}: no decodable audio track found", path.display()))?
        .clone();
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| anyhow::anyhow!("{}: unsupported codec ({e})", path.display()))?;

    let mut mono_samples: Vec<f32> = Vec::new();
    let mut native_rate: Option<u32> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymErr::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(anyhow::anyhow!("{}: read error: {e}", path.display())),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymErr::IoError(_)) | Err(SymErr::DecodeError(_)) => continue, // skip one bad packet, keep going
            Err(e) => return Err(anyhow::anyhow!("{}: decode error: {e}", path.display())),
        };
        let spec = *decoded.spec();
        native_rate.get_or_insert(spec.rate);

        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buf.copy_interleaved_ref(decoded);
        mono_samples.extend(downmix_to_mono(buf.samples(), spec.channels.count()));
    }

    let rate = native_rate
        .ok_or_else(|| anyhow::anyhow!("{}: no audio packets decoded", path.display()))?;
    Ok((mono_samples, rate))
}

/// Resample mono f32 PCM from `native_rate` to `WHISPER_INPUT_RATE` via
/// `rubato` (ADR-0006). A no-op (returns the input unchanged) when the rates
/// already match, so callers can call this unconditionally.
fn resample_to_16k(samples: &[f32], native_rate: u32) -> anyhow::Result<Vec<f32>> {
    use rubato::{FftFixedIn, Resampler};

    if native_rate == WHISPER_INPUT_RATE || samples.is_empty() {
        return Ok(samples.to_vec());
    }

    const CHUNK_SIZE_IN: usize = 1024;
    let mut resampler = FftFixedIn::<f32>::new(
        native_rate as usize,
        WHISPER_INPUT_RATE as usize,
        CHUNK_SIZE_IN,
        2,
        1,
    )
    .map_err(|e| {
        anyhow::anyhow!("resampler init ({native_rate} Hz -> {WHISPER_INPUT_RATE} Hz): {e}")
    })?;

    let expected_len =
        (samples.len() as u64 * WHISPER_INPUT_RATE as u64 / native_rate as u64) as usize;
    let mut output: Vec<f32> = Vec::with_capacity(expected_len + CHUNK_SIZE_IN);

    let mut pos = 0usize;
    while pos + CHUNK_SIZE_IN <= samples.len() {
        let input = [samples[pos..pos + CHUNK_SIZE_IN].to_vec()];
        let out = resampler
            .process(&input, None)
            .map_err(|e| anyhow::anyhow!("resample chunk: {e}"))?;
        output.extend_from_slice(&out[0]);
        pos += CHUNK_SIZE_IN;
    }

    // Final partial chunk (may be shorter than CHUNK_SIZE_IN, or empty if
    // samples.len() was an exact multiple).
    let tail = samples[pos..].to_vec();
    let tail_input = [tail];
    let tail_ref: Option<&[Vec<f32>]> = if tail_input[0].is_empty() {
        None
    } else {
        Some(&tail_input)
    };
    let out = resampler
        .process_partial(tail_ref, None)
        .map_err(|e| anyhow::anyhow!("resample tail: {e}"))?;
    output.extend_from_slice(&out[0]);

    // Flush the resampler's internal delay. Bounded (never unconditional) —
    // an FFT resampler's remaining buffer empties in one or two calls; this
    // caps it well above that so a future rubato behaviour change fails
    // loud (a truncated transcript) rather than hanging (JSOC: bounded
    // execution).
    for _ in 0..8 {
        let out = resampler
            .process_partial::<Vec<f32>>(None, None)
            .map_err(|e| anyhow::anyhow!("resample flush: {e}"))?;
        if out[0].is_empty() {
            break;
        }
        output.extend_from_slice(&out[0]);
    }

    // Trim the resampler's start-up delay, then the length to exactly what
    // the input duration and rate ratio predict (README's documented
    // "resampling a clip" procedure).
    let delay = resampler.output_delay();
    if delay < output.len() {
        output.drain(0..delay);
    } else {
        output.clear();
    }
    output.truncate(expected_len);
    Ok(output)
}

/// Decode any of `.wav`, `.mp3`, `.m4a`, `.aac`, `.flac` at any input sample
/// rate into the mono 16 kHz f32 PCM whisper expects (`--file` batch mode,
/// TASK-VTT130). WAV stays on `hound` (ADR-0006); everything else goes
/// through `symphonia`. Both paths feed the same `resample_to_16k` step, so
/// a WAV recorded at, say, 44.1 kHz now works too — the old "must be
/// 16 kHz" restriction was specific to `decode_wav_to_samples`
/// (`RetranscribeLast`'s contract, kept unchanged), not to `--file` itself.
pub fn decode_audio_to_samples(path: &Path) -> anyhow::Result<Vec<f32>> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let (samples, native_rate) = if ext == "wav" {
        decode_wav_native(path)?
    } else {
        decode_via_symphonia(path)?
    };
    resample_to_16k(&samples, native_rate)
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

    // ── Long-dictation chunking and the repetition guard (TASK-VTT179) ───────

    const RATE: usize = WHISPER_INPUT_RATE as usize;

    /// `secs` of a steady 220 Hz tone with silent half-second gaps starting
    /// at each of `gaps_at` seconds.
    fn tone_with_gaps(secs: usize, gaps_at: &[f32]) -> Vec<f32> {
        (0..secs * RATE)
            .map(|i| {
                let t = i as f32 / RATE as f32;
                if gaps_at.iter().any(|&g| t >= g && t < g + 0.5) {
                    0.0
                } else {
                    0.5 * (2.0 * std::f32::consts::PI * 220.0 * t).sin()
                }
            })
            .collect()
    }

    fn production_ranges(samples: &[f32]) -> Vec<(usize, usize)> {
        pause_aligned_ranges(
            samples,
            MAX_CHUNK_SECS * RATE,
            CUT_SLACK_SECS * RATE,
            QUIET_WINDOW_MS * RATE / 1000,
        )
    }

    fn assert_contiguous_cover(ranges: &[(usize, usize)], total: usize, max_len: usize) {
        assert_eq!(ranges.first().map(|r| r.0), Some(0), "starts at 0");
        assert_eq!(
            ranges.last().map(|r| r.1),
            Some(total),
            "ends at the last sample"
        );
        for pair in ranges.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "ranges are contiguous: {ranges:?}");
        }
        for &(s, e) in ranges {
            assert!(e > s, "no empty range: {ranges:?}");
            assert!(e - s <= max_len, "range {s}..{e} longer than {max_len}");
        }
    }

    #[test]
    fn short_and_empty_audio_stay_one_range() {
        assert_eq!(production_ranges(&[]), vec![(0, 0)]);
        let ten_secs = tone_with_gaps(10, &[]);
        assert_eq!(production_ranges(&ten_secs), vec![(0, 10 * RATE)]);
        let exactly_max = tone_with_gaps(MAX_CHUNK_SECS, &[]);
        assert_eq!(
            production_ranges(&exactly_max).len(),
            1,
            "a full chunk is not split"
        );
    }

    #[test]
    fn long_audio_is_cut_inside_the_pauses() {
        let gaps = [23.0f32, 47.0, 71.0, 86.0];
        let samples = tone_with_gaps(100, &gaps);
        let ranges = production_ranges(&samples);
        assert_contiguous_cover(&ranges, samples.len(), MAX_CHUNK_SECS * RATE);
        assert_eq!(
            ranges.len(),
            gaps.len() + 1,
            "one cut per pause: {ranges:?}"
        );
        for (&(_, cut), &gap) in ranges.iter().zip(gaps.iter()) {
            let at = cut as f32 / RATE as f32;
            assert!(
                at >= gap && at <= gap + 0.5,
                "cut at {at:.2}s is outside the pause at {gap}s"
            );
        }
    }

    #[test]
    fn audio_without_pauses_is_still_bounded_and_covered() {
        let samples = tone_with_gaps(61, &[]);
        let ranges = production_ranges(&samples);
        assert_contiguous_cover(&ranges, samples.len(), MAX_CHUNK_SECS * RATE);
        assert_eq!(ranges.len(), 3, "61 s needs three chunks of at most 28 s");
    }

    #[test]
    fn looping_chunk_pieces_fit_the_piece_length() {
        let samples = tone_with_gaps(MAX_CHUNK_SECS, &[9.0, 19.0]);
        let ranges = pause_aligned_ranges(
            &samples,
            PIECE_SECS * RATE,
            PIECE_SLACK_SECS * RATE,
            QUIET_WINDOW_MS * RATE / 1000,
        );
        assert_contiguous_cover(&ranges, samples.len(), PIECE_SECS * RATE);
        assert_eq!(ranges.len(), 3, "{ranges:?}");
    }

    #[test]
    fn loop_counts_consecutive_repeats_ignoring_case_and_punctuation() {
        assert_eq!(longest_loop(""), 0);
        assert_eq!(longest_loop("One. Two. Three."), 1);
        assert_eq!(longest_loop("The cat sat. The cat sat. The cat sat."), 3);
        assert_eq!(
            longest_loop("Hello there. hello there! HELLO, THERE? Something else."),
            3
        );
        assert_eq!(longest_loop("Thanks. Thanks. Next thing."), 2);
        assert_eq!(
            longest_loop("A b. C d. E f. A b. G h."),
            1,
            "only back-to-back repeats count"
        );
        assert_eq!(longest_loop("no punctuation at all"), 1);
        assert_eq!(
            longest_loop("It costs 3.5 pounds. It costs 3.5 pounds."),
            2,
            "a decimal point does not end a sentence"
        );
        assert_eq!(
            longest_loop("... ... ... ..."),
            1,
            "punctuation-only never matches"
        );
    }

    #[test]
    fn loop_counts_a_repeating_block_of_sentences() {
        // whisper repeats whole segments, and a segment can hold two sentences.
        assert_eq!(
            longest_loop("Red one. Blue two. Red one. Blue two. Red one. Blue two."),
            3
        );
        assert_eq!(
            longest_loop("A x. B y. C z. A x. B y. C z. A x. B y. C z. End."),
            3
        );
        assert_eq!(
            longest_loop("Red one. Blue two. Red one. Blue two. Red one."),
            2
        );
        assert_eq!(
            longest_loop("P q. R s. T u. V w. P q. R s. T u. V w. P q. R s. T u. V w."),
            1,
            "blocks longer than MAX_LOOP_PERIOD are not looked for"
        );
    }

    #[test]
    fn collapse_keeps_one_copy_of_a_loop_and_leaves_the_rest() {
        let looped = "Start here. Loop line. Loop line. Loop line. Loop line. End here.";
        let (text, removed) = collapse_loops(looped, LOOP_REPEATS);
        assert_eq!(text, "Start here. Loop line. End here.");
        assert_eq!(removed, 3);
        assert_eq!(longest_loop(&text), 1);

        let block = "Start. Red one. Blue two. Red one. Blue two. Red one. Blue two. End.";
        let (text, removed) = collapse_loops(block, LOOP_REPEATS);
        assert_eq!(text, "Start. Red one. Blue two. End.");
        assert_eq!(removed, 4);
    }

    #[test]
    fn collapse_leaves_short_runs_and_clean_text_byte_for_byte() {
        let doubled = "Thanks.  Thanks.\nNext thing";
        assert_eq!(
            collapse_loops(doubled, LOOP_REPEATS),
            (doubled.to_string(), 0)
        );
        assert_eq!(collapse_loops("", LOOP_REPEATS), (String::new(), 0));
    }

    #[test]
    fn join_trimmed_skips_blank_pieces() {
        let parts = vec![
            " a ".to_string(),
            "  ".to_string(),
            String::new(),
            "b".to_string(),
        ];
        assert_eq!(join_trimmed(&parts), "a b");
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

        assert_transcript_has_expected_content(&text);
    }

    /// Shared assertion for both e2e tests below: the fixture's known
    /// content words and spoken digits, however whisper happens to render
    /// them, must be present.
    fn assert_transcript_has_expected_content(text: &str) {
        let normalised: String = text
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect();
        let words: std::collections::HashSet<&str> = normalised.split_whitespace().collect();

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

    #[test]
    #[ignore = "downloads ~142 MB base.en model and runs CPU inference"]
    fn e2e_transcribes_mp3_fixture_via_decode_audio_to_samples() {
        // Same spoken content as testing-one-two-three.wav, transcoded to mp3
        // at 44.1 kHz (a genuinely different container AND sample rate from
        // the 16 kHz wav fixture) — real end-to-end proof for TASK-VTT130's
        // multi-format criterion, not just a decode-only unit test.
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/testing-one-two-three.mp3");
        assert!(fixture.exists(), "missing fixture {}", fixture.display());

        let samples = decode_audio_to_samples(&fixture).expect("decode mp3 fixture");
        assert!(!samples.is_empty(), "mp3 fixture decoded to zero samples");

        let model = ensure_e2e_model();
        let engine = WhisperEngine::new(&model, "base.en").expect("load base.en");
        let text = engine
            .transcribe(&samples, "en", None)
            .expect("transcription should succeed");

        assert_transcript_has_expected_content(&text);
    }
}
