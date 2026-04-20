use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 16000;
const MAX_RECORDING_SECONDS: usize = 300; // 5 minutes
const MIN_DURATION_SECS: f32 = 0.5;
const MIN_AMPLITUDE: i16 = 500;

pub enum RecordingResult {
    Audio { samples: Vec<f32>, path: PathBuf },
    TooShort,
    TooQuiet,
    MaxLength { samples: Vec<f32>, path: PathBuf },
}

type BufferFullCallback = Arc<Mutex<Option<Box<dyn Fn() + Send>>>>;

pub struct Audio {
    _stream: cpal::Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
    recording: Arc<AtomicBool>,
    buffer_full: Arc<AtomicBool>,
    buffer_full_callback: BufferFullCallback,
}

// Safety: cpal::Stream is Send but not marked as such in all versions.
// The stream is created on the main thread and never moved between threads;
// only the Arc-wrapped shared state is accessed from multiple threads.
unsafe impl Send for Audio {}
unsafe impl Sync for Audio {}

impl Audio {
    pub fn new() -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = match host.default_input_device() {
            Some(d) => d,
            None => {
                // Help the user see what cpal can see — the "no default" case is
                // usually a PulseAudio / PipeWire misconfiguration where devices
                // exist but no default source is set. Listing them points at the
                // fix: `pactl set-default-source <source-name>`.
                let available: Vec<String> = host
                    .input_devices()
                    .map(|iter| iter.filter_map(|d| d.name().ok()).collect::<Vec<_>>())
                    .unwrap_or_default();
                if available.is_empty() {
                    anyhow::bail!(
                        "No input audio devices detected. Check that your microphone \
                         is plugged in and recognised by PulseAudio/PipeWire: \
                         run `pactl list sources short` to verify."
                    );
                } else {
                    anyhow::bail!(
                        "No default input device set, but {} input device(s) are available: [{}]. \
                         Set a default with `pactl set-default-source <source-name>`.",
                        available.len(),
                        available.join(", ")
                    );
                }
            }
        };

        crate::vtt_log!(
            "Audio device: {}",
            device.name().unwrap_or_else(|_| "unknown".into())
        );

        let max_samples = SAMPLE_RATE as usize * MAX_RECORDING_SECONDS;
        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(max_samples)));
        let recording = Arc::new(AtomicBool::new(false));
        let buffer_full = Arc::new(AtomicBool::new(false));
        let buffer_full_callback: BufferFullCallback = Arc::new(Mutex::new(None));

        let buf_clone = buffer.clone();
        let rec_clone = recording.clone();
        let full_clone = buffer_full.clone();
        let cb_clone = buffer_full_callback.clone();
        let max = max_samples;

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !rec_clone.load(Ordering::Relaxed) {
                    return;
                }
                let mut buf = match buf_clone.try_lock() {
                    Ok(b) => b,
                    Err(_) => return, // Skip frame if locked (avoids blocking audio thread)
                };

                // compute_append is pure and unit-tested — see its tests in this
                // module for the boundary cases (exact fill, overflow, already full).
                let (take, became_full) = compute_append(buf.len(), data.len(), max);
                if take > 0 {
                    buf.extend_from_slice(&data[..take]);
                }
                if became_full && !full_clone.load(Ordering::Relaxed) {
                    full_clone.store(true, Ordering::Relaxed);
                    if let Ok(cb) = cb_clone.try_lock() {
                        if let Some(ref f) = *cb {
                            f();
                        }
                    }
                }
            },
            |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;
        crate::vtt_log!(
            "Audio stream opened (zero-latency mode, {} Hz)",
            SAMPLE_RATE
        );

        Ok(Audio {
            _stream: stream,
            buffer,
            recording,
            buffer_full,
            buffer_full_callback,
        })
    }

    pub fn set_buffer_full_callback<F: Fn() + Send + 'static>(&self, f: F) {
        *self.buffer_full_callback.lock().unwrap() = Some(Box::new(f));
    }

    pub fn start_recording(&self) {
        let mut buf = self.buffer.lock().unwrap();
        buf.clear();
        self.buffer_full.store(false, Ordering::SeqCst);
        self.recording.store(true, Ordering::SeqCst);
        crate::vtt_log!("Recording started");
    }

    pub fn stop_recording(&self) -> Option<RecordingResult> {
        self.recording.store(false, Ordering::SeqCst);
        let was_full = self.buffer_full.load(Ordering::SeqCst);

        let buf = self.buffer.lock().unwrap();
        let sample_count = buf.len();
        let duration = sample_count as f32 / SAMPLE_RATE as f32;

        // Check minimum duration
        if duration < MIN_DURATION_SECS {
            crate::vtt_log!("Recording too short ({:.2}s)", duration);
            return Some(RecordingResult::TooShort);
        }

        // Check amplitude (convert to i16 range for comparison)
        let max_amp: i16 = buf
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .map(|s| s.abs())
            .max()
            .unwrap_or(0);

        if max_amp < MIN_AMPLITUDE {
            crate::vtt_log!("Audio too quiet (amplitude {})", max_amp);
            return Some(RecordingResult::TooQuiet);
        }

        crate::vtt_log!(
            "Recording stopped: {:.2}s{}, amplitude: {}",
            duration,
            if was_full { " (MAX LENGTH)" } else { "" },
            max_amp
        );

        // Snapshot samples for in-process transcription; also write WAV for the
        // debug recordings archive. If WAV write fails, transcription still proceeds
        // because samples are already in memory.
        let samples: Vec<f32> = buf.clone();
        drop(buf);
        let path = match write_wav(&samples) {
            Ok(p) => {
                crate::vtt_log!("Saved recording to {}", p.display());
                p
            }
            Err(e) => {
                crate::vtt_log!("Failed to write WAV (transcription continues): {}", e);
                PathBuf::new()
            }
        };

        if was_full {
            Some(RecordingResult::MaxLength { samples, path })
        } else {
            Some(RecordingResult::Audio { samples, path })
        }
    }
}

fn write_wav(samples: &[f32]) -> anyhow::Result<PathBuf> {
    let tmp = tempfile::Builder::new()
        .prefix("vtt_recording_")
        .suffix(".wav")
        .tempfile_in("/tmp")?;

    let path = tmp.path().to_path_buf();

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&path, spec)?;
    for &s in samples {
        writer.write_sample((s.clamp(-1.0, 1.0) * 32767.0) as i16)?;
    }
    writer.finalize()?;

    // Keep the temp file (don't auto-delete)
    tmp.keep()?;

    Ok(path)
}

/// Pure: given the current buffer length, an incoming chunk size, and the
/// maximum allowed buffer length, compute how many samples we can append and
/// whether the append causes the buffer to transition to "full".
///
/// Returns `(take, became_full)` where:
/// - `take` is the number of samples to append (0..=incoming, clamped to available space)
/// - `became_full` is true if after the append the buffer reached `max`
///
/// Used by the audio stream callback to avoid overflowing the bounded capture
/// buffer (currently 5 min at 16 kHz = 4.8 M samples). Becoming full flips the
/// `buffer_full` atomic flag and fires the notification callback.
fn compute_append(current_len: usize, incoming: usize, max: usize) -> (usize, bool) {
    let space = max.saturating_sub(current_len);
    let take = incoming.min(space);
    let new_len = current_len + take;
    let became_full = new_len >= max;
    (take, became_full)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_append_room_for_full_chunk() {
        assert_eq!(compute_append(100, 50, 1000), (50, false));
    }

    #[test]
    fn compute_append_chunk_exactly_fills_buffer() {
        assert_eq!(compute_append(900, 100, 1000), (100, true));
    }

    #[test]
    fn compute_append_chunk_overflows_gets_clamped() {
        assert_eq!(
            compute_append(950, 100, 1000),
            (50, true),
            "only 50 of the 100 fit, buffer transitions to full"
        );
    }

    #[test]
    fn compute_append_already_full_takes_nothing() {
        assert_eq!(
            compute_append(1000, 50, 1000),
            (0, true),
            "at capacity, take nothing but still report full"
        );
    }

    #[test]
    fn compute_append_past_full_takes_nothing() {
        // Shouldn't happen in practice but should not panic or wrap.
        assert_eq!(compute_append(2000, 50, 1000), (0, true));
    }

    #[test]
    fn compute_append_empty_buffer_empty_chunk() {
        assert_eq!(compute_append(0, 0, 1000), (0, false));
    }

    #[test]
    fn compute_append_empty_buffer_full_chunk() {
        assert_eq!(compute_append(0, 1000, 1000), (1000, true));
    }

    #[test]
    fn compute_append_realistic_audio_frame() {
        // 16 kHz, 5 min max buffer, 10 ms frame = 160 samples
        let max = 16_000 * 300;
        let (take, full) = compute_append(max - 100, 160, max);
        assert_eq!(take, 100, "only 100 samples fit in the last slice");
        assert!(full, "buffer is at capacity after the append");
    }
}
