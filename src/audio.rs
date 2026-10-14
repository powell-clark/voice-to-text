use crate::segment::{Policy, Segmenter};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

/// Whisper's required input rate. Every supported Whisper model is trained at
/// 16 kHz, so the samples handed to the engine are always at this rate — but
/// capture no longer happens here. `stop_recording` resamples the capture
/// buffer down to this immediately before it leaves the audio layer, so the
/// transcription path sees exactly what it saw before TASK-VTT150.
const WHISPER_SAMPLE_RATE: u32 = crate::whisper::WHISPER_INPUT_RATE;
/// The rate we ask the capture device for. 48 kHz is the native rate of every
/// modern interface and the rate speech synthesis training wants; capturing at
/// 16 kHz threw away detail that cannot be recovered by upsampling later. The
/// device may refuse, in which case the native-format fallback records whatever
/// rate the device does offer and `capture_rate` carries it.
const CAPTURE_SAMPLE_RATE: u32 = 48000;
/// Most audio the capture callback's buffer holds before the segment tick
/// moves it on. The tick drains it every second, so this is reached only if
/// that thread stalls for a minute: it bounds memory, not how long a hold can
/// last, which is unlimited now that a hold is cut into segments (TASK-VTT195).
const LIVE_BUFFER_MAX_SECS: usize = 60;
/// What the callback's buffer is sized for after each drain, so an ordinary
/// tick never has the audio thread reallocate.
const LIVE_BUFFER_PREALLOC: usize = CAPTURE_SAMPLE_RATE as usize * 4;
/// Most segments one tick hands on. Each takes at least a threshold's worth
/// of audio, so a tick never finds more than one due unless it stalled.
const MAX_CUTS_PER_TICK: usize = 64;
/// Minimum recording duration. Anything shorter returns `TooShort` and
/// is discarded silently (avoids transcribing 0.1s button-smash presses).
const MIN_DURATION_SECS: f32 = 0.5;
/// Minimum peak amplitude (in i16 range, peak = 32767). Below this we
/// assume silence/background and return `TooQuiet` rather than wasting
/// GPU cycles on Whisper detecting hallucinated speech.
const MIN_AMPLITUDE: i16 = 500;

/// Where a recording sits in its hold. A hold under the segment threshold is
/// a single segment, both first and last; a longer hold is cut into several
/// while the key stays down (TASK-VTT195).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Part {
    /// Position in the hold, from 0.
    pub index: u32,
    /// The segment the release ended. Its text is typed together with every
    /// earlier segment's, in order; nothing is typed before it.
    pub last: bool,
}

impl Part {
    /// A recording that is a whole hold on its own.
    pub const WHOLE: Part = Part {
        index: 0,
        last: true,
    };
}

/// Result of one push-to-talk recording cycle, or of one segment cut from a
/// long hold. The worker pattern-matches on this to decide between
/// transcribing and silently resetting the tray to Ready.
pub enum RecordingResult {
    /// A segment that passed the duration + amplitude thresholds. Samples
    /// and archived WAV path both present.
    Audio {
        samples: Vec<f32>,
        path: PathBuf,
        native: Option<NativeCapture>,
        part: Part,
    },
    /// Recording was below MIN_DURATION_SECS. Discard.
    TooShort,
    /// Recording was above MIN_DURATION but peak amplitude below MIN_AMPLITUDE.
    /// Probably silence or a hot mic with no speech. Discard.
    TooQuiet,
    /// The key came up after segments had already been cut, and what followed
    /// the last cut was too short or too quiet to transcribe. The earlier
    /// segments are the whole hold, and their text is still delivered.
    NoFinalSegment,
    /// The stream delivered zero samples across a hold long enough to have
    /// captured some — the device re-enumerated, the audio session restarted,
    /// or the source was suspended underneath us. Distinct from TooShort so
    /// the user learns their mic is the problem, not their timing; a tap too
    /// brief for the first callback to fire is TooShort, not this. The stream has already
    /// been re-opened against the current default device by the time the
    /// caller sees this, so the next press should capture normally.
    NoAudioCaptured,
}

/// The capture buffer at the device's own rate, retained only when recording
/// archiving is switched on. This is the single place native-rate samples
/// survive past the audio layer: with archiving off it is always `None`, so a
/// build with the setting absent never holds a second copy of the audio and
/// never has anything to write.
pub struct NativeCapture {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

/// Pure classification of a finished capture, separated from `stop_recording`
/// so the empty-vs-short-vs-quiet boundaries are unit-testable without a
/// live audio device.
#[derive(Debug, PartialEq, Eq)]
enum CaptureClass {
    /// 0 samples despite a long enough hold — the stream is dead.
    Empty,
    /// Real samples but under MIN_DURATION_SECS.
    TooShort,
    /// Long enough but peak amplitude under MIN_AMPLITUDE.
    TooQuiet,
    /// Normal recording.
    Normal,
}

fn classify_capture(sample_count: usize, max_amp: i16, held_secs: f32, rate: u32) -> CaptureClass {
    if sample_count == 0 {
        // Zero samples means a dead stream only when the key was held long
        // enough that cpal should have delivered a buffer. A tap shorter than
        // that can legitimately end before the first callback fires, and
        // calling it a dead microphone puts a sticky error icon in the tray
        // and re-opens the stream for nothing (TASK-VTT148).
        if held_secs < MIN_DURATION_SECS {
            return CaptureClass::TooShort;
        }
        return CaptureClass::Empty;
    }
    let duration = sample_count as f32 / rate as f32;
    if duration < MIN_DURATION_SECS {
        return CaptureClass::TooShort;
    }
    if max_amp < MIN_AMPLITUDE {
        return CaptureClass::TooQuiet;
    }
    CaptureClass::Normal
}

/// Peak amplitude in i16 range, 0 for no samples.
fn peak_amplitude(samples: &[f32]) -> i16 {
    samples
        .iter()
        .map(|&s| ((s.clamp(-1.0, 1.0) * 32767.0) as i16).abs())
        .max()
        .unwrap_or(0)
}

/// Lock a mutex whose data stays valid even if a holder panicked: a sample
/// buffer or a hold's pending audio is still worth delivering.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// One hold, from the press to the release.
struct Hold {
    segmenter: Segmenter,
    /// Segments handed on for transcription so far; the next one's index.
    sent: u32,
    /// Segments cut, including any too quiet to hand on.
    cuts: u32,
    /// Samples cut off into segments so far.
    cut_samples: usize,
}

pub struct Audio {
    /// The live cpal stream, or None between a death and a successful
    /// re-open. Mutex because recovery replaces it from the hotkey thread.
    stream: Mutex<Option<cpal::Stream>>,
    /// Where the capture callback appends. The segment tick swaps it out
    /// every second, so it only ever holds the last second or so.
    buffer: Arc<Mutex<Vec<f32>>>,
    recording: Arc<AtomicBool>,
    /// Samples the callback has appended since the press, counted so the
    /// release can check every one of them reached a segment.
    captured: Arc<AtomicUsize>,
    /// The hold in progress, None between holds. Serialises the segment
    /// tick against the release.
    hold: Mutex<Option<Hold>>,
    /// Seconds of audio after which a held key's capture is cut at the next
    /// pause (`segment::THRESHOLD_SECS` unless overridden for a test).
    segment_secs: f32,
    /// Set by the stream error callback (audio thread) when the device goes
    /// away; cleared by `try_reopen`. Checked at every `start_recording` so
    /// a dead stream heals before the recording instead of after it fails.
    stream_dead: Arc<AtomicBool>,
    /// User-selected input-device ordinal (settings `device=N`), or `None` to
    /// track the system default. Held so `try_reopen` rebuilds against the
    /// same chosen device rather than silently reverting to default.
    device_index: Option<usize>,
    /// When the current hold began. A zero-sample capture only means the
    /// stream is dead if the key was held long enough for cpal to have
    /// delivered a buffer — otherwise it just means the user was quick.
    hold_start: Mutex<Option<Instant>>,
    /// The rate the open stream actually delivers, written when the stream is
    /// built. 48 kHz on the direct path; the device's native rate when it
    /// refused 48 kHz. Read by `stop_recording` to compute duration and to tag
    /// the archived audio — never assumed.
    capture_rate: Arc<AtomicU32>,
    /// Mirrors the `archive` setting. Gates whether the native-rate buffer is
    /// retained past `stop_recording` at all.
    archive_enabled: Arc<AtomicBool>,
    /// Mirrors the `denoise` setting. Gates the high-pass on the Whisper path;
    /// the archive is never filtered either way (TASK-VTT145).
    denoise_enabled: Arc<AtomicBool>,
}

/// Resolve a saved input-device ordinal against the number of devices cpal
/// currently reports. Returns `Some(ordinal)` when the index addresses a real
/// device, or `None` when it is out of range — the caller then falls back to
/// the default device (with a warning). The `< 0` "no selection" sentinel is
/// filtered to `None` before this point, so reaching here with `None` result
/// always means an out-of-range selection worth warning about.
fn resolve_device_ordinal(index: usize, count: usize) -> Option<usize> {
    (index < count).then_some(index)
}

/// Input device names in the same cpal-reported order `open_capture_stream`
/// indexes by — a device picker built from this list can save an ordinal that
/// `resolve_device_ordinal` will resolve back to the same device (TASK-VTT051).
/// Empty on enumeration failure rather than erroring — a picker UI degrades to
/// "no devices listed", not a crash.
pub fn list_input_device_names() -> Vec<String> {
    cpal::default_host()
        .input_devices()
        .map(|iter| iter.filter_map(|d| d.name().ok()).collect())
        .unwrap_or_default()
}

// Safety: cpal::Stream is Send but not marked as such in all versions.
// The stream is created on the main thread and, on recovery, dropped and
// re-created from the hotkey monitor thread — it is only ever owned and
// driven through the Mutex above, never aliased across threads. On the
// backends we ship (ALSA, WASAPI, CoreAudio) moving the owning handle
// between threads is sound; all other shared state is Arc-wrapped.
unsafe impl Send for Audio {}
unsafe impl Sync for Audio {}

impl Audio {
    /// Open the default input device and start a cpal stream that fills the
    /// internal buffer when `start_recording()` has been called.
    ///
    /// Fails with a helpful error if no default input device exists:
    /// distinguishes "no devices at all" (hardware/driver missing) from
    /// "devices exist but no default set" (PulseAudio misconfiguration,
    /// with `pactl set-default-source` hint).
    ///
    /// The stream is driven by cpal on its own high-priority audio thread.
    /// Calling code only interacts with `start_recording`, `take_segment`
    /// and `stop_recording`.
    /// `device_index` is the user's chosen input-device ordinal (settings
    /// `device=N`), or `None` to follow the system default. An out-of-range
    /// choice logs a warning and falls back to the default rather than failing.
    pub fn new(device_index: Option<usize>) -> anyhow::Result<Self> {
        let buffer: Arc<Mutex<Vec<f32>>> =
            Arc::new(Mutex::new(Vec::with_capacity(LIVE_BUFFER_PREALLOC)));
        let recording = Arc::new(AtomicBool::new(false));
        let captured = Arc::new(AtomicUsize::new(0));
        let stream_dead = Arc::new(AtomicBool::new(false));
        let capture_rate = Arc::new(AtomicU32::new(CAPTURE_SAMPLE_RATE));

        let stream = Self::open_capture_stream(
            &buffer,
            &recording,
            &captured,
            &stream_dead,
            device_index,
            &capture_rate,
        )?;

        Ok(Audio {
            hold_start: Mutex::new(None),
            stream: Mutex::new(Some(stream)),
            buffer,
            recording,
            captured,
            hold: Mutex::new(None),
            segment_secs: crate::segment::live_threshold_secs(),
            stream_dead,
            device_index,
            capture_rate,
            archive_enabled: Arc::new(AtomicBool::new(false)),
            denoise_enabled: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Switch retention of the native-rate capture buffer on or off. Called
    /// once at startup from the `archive` setting, and again if the setting
    /// changes. Off (the default) means `stop_recording` never clones the
    /// native buffer and `RecordingResult::native` is always `None`.
    pub fn set_archive_enabled(&self, on: bool) {
        self.archive_enabled.store(on, Ordering::SeqCst);
    }

    /// Switch low-frequency noise suppression on the transcription path on or
    /// off. On by default; `denoise=0` in settings restores the unfiltered
    /// path exactly.
    pub fn set_denoise_enabled(&self, on: bool) {
        self.denoise_enabled.store(on, Ordering::SeqCst);
    }

    /// Drop the current stream (if any) and open a fresh one against the
    /// current default input device. This is the recovery path for all three
    /// observed stream-death triggers: PipeWire idle-suspend, USB
    /// re-enumeration, and audio-session restart on logout/login — in every
    /// case the held stream is bound to a node that no longer delivers frames
    /// and only a fresh connection recovers (TASK-VTT121).
    ///
    /// Returns true on success. On failure the dead flag stays set so the
    /// next `start_recording` retries.
    fn try_reopen(&self) -> bool {
        // Drop the old stream before opening the new one — a dead-but-held
        // handle can keep the ALSA/WASAPI device claimed.
        *self.stream.lock().unwrap() = None;
        self.stream_dead.store(false, Ordering::SeqCst);
        match Self::open_capture_stream(
            &self.buffer,
            &self.recording,
            &self.captured,
            &self.stream_dead,
            self.device_index,
            &self.capture_rate,
        ) {
            Ok(s) => {
                *self.stream.lock().unwrap() = Some(s);
                crate::vtt_log!("Audio stream re-opened against selected/default input device");
                true
            }
            Err(e) => {
                self.stream_dead.store(true, Ordering::SeqCst);
                crate::vtt_log!("Audio stream re-open failed: {}", e);
                false
            }
        }
    }

    /// Open (or re-open) a cpal input stream on the current default input
    /// device, wired to the shared capture state. Extracted from `new` so
    /// recovery (`try_reopen`) rebuilds the stream with identical fallback
    /// behaviour, including the 16 kHz-direct → native-format chain.
    ///
    /// Fails with a helpful error if no default input device exists:
    /// distinguishes "no devices at all" (hardware/driver missing) from
    /// "devices exist but no default set" (PulseAudio misconfiguration,
    /// with `pactl set-default-source` hint).
    fn open_capture_stream(
        buffer: &Arc<Mutex<Vec<f32>>>,
        recording: &Arc<AtomicBool>,
        captured: &Arc<AtomicUsize>,
        stream_dead: &Arc<AtomicBool>,
        device_index: Option<usize>,
        capture_rate: &Arc<AtomicU32>,
    ) -> anyhow::Result<cpal::Stream> {
        let host = cpal::default_host();

        // Honour a user-selected input device (settings `device=N`) when it
        // resolves to a real cpal ordinal; otherwise fall through to the
        // default-device path below. Out-of-range or an unreadable device list
        // is a warning, never a hard failure — a USB mic may have been
        // unplugged since the index was saved.
        let selected = device_index.and_then(|idx| {
            let devices: Vec<cpal::Device> = match host.input_devices() {
                Ok(iter) => iter.collect(),
                Err(e) => {
                    crate::vtt_log!("Could not enumerate input devices ({e}); using default");
                    return None;
                }
            };
            match resolve_device_ordinal(idx, devices.len()) {
                Some(ordinal) => devices.into_iter().nth(ordinal),
                None => {
                    crate::vtt_log!(
                        "Selected input device index {idx} is out of range ({} available); \
                         falling back to default",
                        devices.len()
                    );
                    None
                }
            }
        });

        let device = match selected.or_else(|| host.default_input_device()) {
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

        let max = CAPTURE_SAMPLE_RATE as usize * LIVE_BUFFER_MAX_SECS;
        // The error callback is the death signal, not just a log line: cpal
        // fires it when the device disappears or the backend fails, and the
        // flag makes the next start_recording re-open the stream instead of
        // recording silence forever (TASK-VTT121).
        let err_cb = {
            let dead = stream_dead.clone();
            move |err| {
                eprintln!("Audio stream error: {}", err);
                dead.store(true, Ordering::SeqCst);
            }
        };
        // Owned clones so the per-callback clones below stay unchanged.
        let (buffer, recording, captured) = (buffer.clone(), recording.clone(), captured.clone());

        // Attempt 1: ask the device for 48 kHz mono f32 directly. ALSA (Linux)
        // accepts this, and 48 kHz is the native rate of essentially every
        // interface, so this path now captures at the device's real resolution
        // instead of asking ALSA to throw two thirds of it away.
        let preferred = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(CAPTURE_SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = {
            let (b, r, n) = (buffer.clone(), recording.clone(), captured.clone());
            match device.build_input_stream(
                &preferred,
                move |data: &[f32], _: &cpal::InputCallbackInfo| ingest(&b, &r, &n, max, data),
                err_cb.clone(),
                None,
            ) {
                Ok(s) => {
                    capture_rate.store(CAPTURE_SAMPLE_RATE, Ordering::SeqCst);
                    crate::vtt_log!(
                        "Audio stream opened (native {} Hz mono)",
                        CAPTURE_SAMPLE_RATE
                    );
                    s
                }
                // Attempt 2: WASAPI shared mode (Windows) rejects any non-native
                // format. Open the device's native config and downmix + resample
                // to 16 kHz mono in the capture callback.
                Err(direct_err) => {
                    let native = device.default_input_config().map_err(|e| {
                        anyhow::anyhow!(
                            "no native input config ({e}); {CAPTURE_SAMPLE_RATE} Hz direct also failed: {direct_err}"
                        )
                    })?;
                    let rate = native.sample_rate().0;
                    let channels = native.channels() as usize;
                    let fmt = native.sample_format();
                    let cfg: StreamConfig = native.into();
                    capture_rate.store(rate, Ordering::SeqCst);
                    crate::vtt_log!(
                        "{} Hz mono unsupported ({direct_err}); capturing native {} Hz {} ch {:?}",
                        CAPTURE_SAMPLE_RATE,
                        rate,
                        channels,
                        fmt
                    );
                    match fmt {
                        cpal::SampleFormat::F32 => {
                            let (b, r, n) = (buffer.clone(), recording.clone(), captured.clone());
                            device.build_input_stream(
                                &cfg,
                                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                    let mono = downmix_to_mono(data, channels);
                                    ingest(&b, &r, &n, max, &mono);
                                },
                                err_cb.clone(),
                                None,
                            )?
                        }
                        cpal::SampleFormat::I16 => {
                            let (b, r, n) = (buffer.clone(), recording.clone(), captured.clone());
                            device.build_input_stream(
                                &cfg,
                                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                                    let floats: Vec<f32> =
                                        data.iter().map(|&s| s as f32 / 32768.0).collect();
                                    let mono = downmix_to_mono(&floats, channels);
                                    ingest(&b, &r, &n, max, &mono);
                                },
                                err_cb.clone(),
                                None,
                            )?
                        }
                        other => anyhow::bail!(
                            "unsupported input sample format {:?} (device {} Hz {} ch)",
                            other,
                            rate,
                            channels
                        ),
                    }
                }
            }
        };

        stream.play()?;
        crate::vtt_log!(
            "Audio capture started ({} Hz mono)",
            capture_rate.load(Ordering::SeqCst)
        );

        Ok(stream)
    }

    /// Begin a new recording. Empties the buffer, opens a new hold and flips
    /// the recording flag so the cpal callback starts appending samples.
    /// Called on hotkey press.
    ///
    /// Safe to call while a previous recording is still being transcribed
    /// (transcription works from its own segment, not the live buffer).
    pub fn start_recording(&self) {
        // Heal before recording: if the stream died since the last press
        // (device unplugged, session restarted), re-open now so this
        // recording captures instead of coming back empty.
        if self.stream_dead.load(Ordering::SeqCst) {
            crate::vtt_log!("Audio stream flagged dead — re-opening before recording");
            self.try_reopen();
        }
        let rate = self.capture_rate.load(Ordering::SeqCst);
        let policy = Policy::new(rate, self.segment_secs);
        *lock(&self.hold) = Some(Hold {
            segmenter: Segmenter::new(policy),
            sent: 0,
            cuts: 0,
            cut_samples: 0,
        });
        lock(&self.buffer).clear();
        self.captured.store(0, Ordering::SeqCst);
        *lock(&self.hold_start) = Some(Instant::now());
        self.recording.store(true, Ordering::SeqCst);
        crate::vtt_log!(
            "Recording started (segments cut after {:.0}s)",
            policy.threshold_secs()
        );
    }

    /// While the key is held: move the captured audio on to the hold, and
    /// return the next segment once one is due, ready to transcribe. A due
    /// segment is cut at the first pause past the threshold, or at the
    /// quietest stretch by the cap. None between holds and while no cut is
    /// due. A segment with nothing loud enough to be speech is dropped, as a
    /// quiet recording is. Called every second by the segment tick.
    pub fn take_segment(&self) -> Option<RecordingResult> {
        let mut guard = lock(&self.hold);
        let hold = guard.as_mut()?;
        hold.segmenter
            .push(&swap_out(&self.buffer, LIVE_BUFFER_PREALLOC));
        let rate = self.capture_rate.load(Ordering::SeqCst);
        // Bounded: each cut takes at least a threshold's worth of audio, and
        // a tick adds about a second.
        for _ in 0..MAX_CUTS_PER_TICK {
            let samples = hold.segmenter.take_due()?;
            hold.cuts += 1;
            hold.cut_samples += samples.len();
            let secs = samples.len() as f32 / rate as f32;
            let peak = peak_amplitude(&samples);
            if peak < MIN_AMPLITUDE {
                crate::vtt_log!(
                    "Cut {} ({:.2}s) too quiet (amplitude {}); not transcribed",
                    hold.cuts,
                    secs,
                    peak
                );
                continue;
            }
            let part = Part {
                index: hold.sent,
                last: false,
            };
            hold.sent += 1;
            crate::vtt_log!(
                "Segment {} cut ({:.2}s) with the key held; transcribing in the background",
                hold.sent,
                secs
            );
            return Some(self.package(samples, rate, part));
        }
        None
    }

    /// End the current hold and return its last segment: everything since
    /// the last cut, or the whole hold when nothing was cut.
    ///
    /// Returns `None` if no recording was in progress. Otherwise returns
    /// a `RecordingResult` variant:
    /// - `Audio`, marked last, for a segment that passed the duration +
    ///   amplitude thresholds; after a cut, any audible stretch passes,
    ///   however short, since the key was held, not tapped
    /// - `NoFinalSegment` when segments were already handed on and nothing
    ///   audible followed the last cut: the earlier segments stand
    /// - `NoAudioCaptured` if the stream delivered 0 samples across a hold
    ///   long enough to have captured some (dead device/session — the stream
    ///   is re-opened before returning)
    /// - `TooShort` if < MIN_DURATION_SECS (discarded, tray resets)
    /// - `TooQuiet` if peak amplitude < MIN_AMPLITUDE (discarded)
    ///
    /// Also writes the samples to a temp-dir WAV for the debug recordings
    /// archive. If WAV write fails, transcription proceeds from the in-memory
    /// samples — disk is not on the critical path.
    pub fn stop_recording(&self) -> Option<RecordingResult> {
        let mut guard = lock(&self.hold);
        self.recording.store(false, Ordering::SeqCst);
        let rate = self.capture_rate.load(Ordering::SeqCst);
        let live = swap_out(&self.buffer, LIVE_BUFFER_PREALLOC);
        let captured = self.captured.load(Ordering::SeqCst);
        // How long the key was actually down, which is not the same as how
        // much audio arrived — that gap is the whole point of the check.
        let held_secs = lock(&self.hold_start)
            .take()
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);
        let Some(Hold {
            mut segmenter,
            sent,
            cuts,
            cut_samples,
        }) = guard.take()
        else {
            crate::vtt_log!("Key released with no recording in progress");
            return None;
        };
        drop(guard);
        segmenter.push(&live);
        let samples = segmenter.finish();
        let sample_count = samples.len();
        let duration = sample_count as f32 / rate as f32;
        let max_amp = peak_amplitude(&samples);

        if cuts > 0 {
            // Every sample the callback appended belongs to a segment: one
            // cut while the key was held, or this last one.
            let segmented = cut_samples + sample_count;
            crate::vtt_log!(
                "Hold ended after {} cut(s): {} samples captured, {} in segments, {} unaccounted",
                cuts,
                captured,
                segmented,
                captured.abs_diff(segmented)
            );
            if max_amp < MIN_AMPLITUDE {
                crate::vtt_log!(
                    "Nothing audible after the last cut ({:.2}s, amplitude {})",
                    duration,
                    max_amp
                );
                return Some(if sent > 0 {
                    RecordingResult::NoFinalSegment
                } else {
                    RecordingResult::TooQuiet
                });
            }
        } else {
            match classify_capture(sample_count, max_amp, held_secs, rate) {
                CaptureClass::Empty => {
                    // Zero samples after a hold long enough to have filled a
                    // buffer is a dead stream. Say so, and re-open now so the
                    // next press works without a manual restart (TASK-VTT121
                    // zero-frame watchdog). A hasty tap never reaches here —
                    // it classifies as TooShort above (TASK-VTT148).
                    crate::vtt_log!("No audio captured (0 samples) — re-opening capture stream");
                    self.try_reopen();
                    return Some(RecordingResult::NoAudioCaptured);
                }
                CaptureClass::TooShort => {
                    crate::vtt_log!("Recording too short ({:.2}s)", duration);
                    return Some(RecordingResult::TooShort);
                }
                CaptureClass::TooQuiet => {
                    crate::vtt_log!("Audio too quiet (amplitude {})", max_amp);
                    return Some(RecordingResult::TooQuiet);
                }
                CaptureClass::Normal => {}
            }
        }

        crate::vtt_log!(
            "Recording stopped: {:.2}s, amplitude: {}{}",
            duration,
            max_amp,
            if sent > 0 {
                format!(" (segment {} of the hold)", sent + 1)
            } else {
                String::new()
            }
        );
        Some(self.package(
            samples,
            rate,
            Part {
                index: sent,
                last: true,
            },
        ))
    }

    /// Ready one segment, at the capture rate, for the worker: the 16 kHz
    /// copy Whisper reads, the debug WAV of that copy, and the native-rate
    /// original when archiving is on.
    fn package(&self, captured: Vec<f32>, rate: u32, part: Part) -> RecordingResult {
        // Resample to Whisper's rate here, once, on the finished segment. The
        // debug recordings archive is written from these samples too, so the
        // re-transcribe-last recovery net keeps reading the 16 kHz wavs it has
        // always read.
        let mut samples: Vec<f32> = resample_to_whisper(&captured, rate);

        // Rumble suppression, on the transcription path only. `native` below
        // is the untouched capture, so the voice-clone corpus keeps the
        // original audio while Whisper gets the cleaned version (TASK-VTT145).
        if self.denoise_enabled.load(Ordering::SeqCst) {
            samples = crate::denoise::suppress_rumble(&samples, WHISPER_SAMPLE_RATE);
        }
        let path = match write_wav(&samples, WHISPER_SAMPLE_RATE) {
            Ok(p) => {
                crate::vtt_log!("Saved recording to {}", p.display());
                p
            }
            Err(e) => {
                crate::vtt_log!("Failed to write WAV (transcription continues): {}", e);
                PathBuf::new()
            }
        };

        // Retain the native-rate capture only when archiving is on. This is the
        // one branch that decides whether a copy of the audio at the device's
        // rate outlives this call, so with the setting absent the memory and
        // the disk behaviour are exactly what they were before TASK-VTT150.
        let native = self
            .archive_enabled
            .load(Ordering::SeqCst)
            .then_some(NativeCapture {
                samples: captured,
                sample_rate: rate,
            });

        RecordingResult::Audio {
            samples,
            path,
            native,
            part,
        }
    }
}

pub(crate) fn write_wav(samples: &[f32], sample_rate: u32) -> anyhow::Result<PathBuf> {
    // Use the platform temp dir, not a hardcoded "/tmp" — on Windows "/tmp"
    // resolves to a non-existent "C:\tmp" and the write fails (debug recordings
    // silently lost, warning logged every transcription). temp_dir() returns
    // "/tmp" on Linux (unless $TMPDIR overrides) so Linux behaviour is unchanged.
    let tmp = tempfile::Builder::new()
        .prefix("vtt_recording_")
        .suffix(".wav")
        .tempfile_in(std::env::temp_dir())?;

    let path = tmp.path().to_path_buf();

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
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

/// Take everything the capture callback has appended, leaving it an empty
/// buffer sized for `capacity`. The allocation happens before the lock, so
/// the lock is held for a pointer swap and the audio thread waits on nothing.
fn swap_out(buffer: &Mutex<Vec<f32>>, capacity: usize) -> Vec<f32> {
    let fresh = Vec::with_capacity(capacity);
    std::mem::replace(&mut *lock(buffer), fresh)
}

/// Append captured mono f32 samples, at the device's own rate, into the shared
/// recording buffer, honouring the recording flag and the buffer's memory
/// bound, and count them. Shared by the direct and the native-format capture
/// callbacks.
///
/// Waits for the lock rather than skipping the frame. Every other holder
/// keeps it only for a pointer swap or a clear, so the wait is short, while a
/// skipped frame is speech lost — and with the segment tick swapping the
/// buffer every second, a skip would land on a cut (TASK-VTT195).
fn ingest(
    buffer: &Mutex<Vec<f32>>,
    recording: &AtomicBool,
    captured: &AtomicUsize,
    max: usize,
    data: &[f32],
) {
    if !recording.load(Ordering::Relaxed) {
        return;
    }
    let mut buf = lock(buffer);
    let take = compute_append(buf.len(), data.len(), max);
    buf.extend_from_slice(&data[..take]);
    captured.fetch_add(take, Ordering::Relaxed);
}

/// Average interleaved frames down to a single mono channel. `channels <= 1`
/// returns the input unchanged (already mono).
fn downmix_to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    data.chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect()
}

/// Linear-interpolation resample from `in_rate` to Whisper's 16 kHz. Applied
/// once, to the whole finished capture, immediately before the samples leave
/// the audio layer for transcription — not per callback, so there are no chunk
/// boundary artefacts at all now. `in_rate == 16000` or empty input returns a
/// copy.
fn resample_to_whisper(input: &[f32], in_rate: u32) -> Vec<f32> {
    if in_rate == WHISPER_SAMPLE_RATE || input.is_empty() {
        return input.to_vec();
    }
    let ratio = WHISPER_SAMPLE_RATE as f64 / in_rate as f64;
    let out_len = ((input.len() as f64) * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    let last = input.len() - 1;
    for i in 0..out_len {
        let src = i as f64 / ratio;
        let idx = src as usize;
        let frac = (src - idx as f64) as f32;
        let a = input[idx.min(last)];
        let b = input[(idx + 1).min(last)];
        out.push(a + (b - a) * frac);
    }
    out
}

/// Pure: given the current buffer length, an incoming chunk size, and the
/// maximum allowed buffer length, how many samples can be appended
/// (0..=incoming, clamped to the space left).
///
/// Used by the audio stream callback to bound the live buffer, which the
/// segment tick normally empties every second (LIVE_BUFFER_MAX_SECS).
fn compute_append(current_len: usize, incoming: usize, max: usize) -> usize {
    incoming.min(max.saturating_sub(current_len))
}

/// A deviceless capture for tests here and in the worker's tests: the
/// capture callback and the segment tick are played by hand, silently,
/// with synthetic audio and no device, sound server or display.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// An `Audio` with no device behind it, so a test can play the capture
    /// callback and the segment tick by hand. Archiving is on, so every
    /// segment carries its audio at the capture rate for checking.
    pub(crate) fn deviceless(segment_secs: f32) -> Audio {
        Audio {
            stream: Mutex::new(None),
            buffer: Arc::new(Mutex::new(Vec::new())),
            recording: Arc::new(AtomicBool::new(false)),
            captured: Arc::new(AtomicUsize::new(0)),
            hold: Mutex::new(None),
            segment_secs,
            stream_dead: Arc::new(AtomicBool::new(false)),
            device_index: None,
            hold_start: Mutex::new(None),
            capture_rate: Arc::new(AtomicU32::new(CAPTURE_SAMPLE_RATE)),
            archive_enabled: Arc::new(AtomicBool::new(true)),
            denoise_enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Dictation at the capture rate: 1.5 s phrases, each followed by half a
    /// second of near-silence.
    pub(crate) fn phrases(secs: usize) -> Vec<f32> {
        let rate = CAPTURE_SAMPLE_RATE as usize;
        (0..secs * rate)
            .map(|i| {
                if i % (2 * rate) < rate * 3 / 2 {
                    0.3 * (i as f32 * 0.05).sin()
                } else {
                    0.0005 * ((i % 7) as f32 - 3.0) / 3.0
                }
            })
            .collect()
    }

    /// Deliver one capture-callback frame of `samples`, as cpal would.
    pub(crate) fn feed(dev: &Audio, samples: &[f32]) {
        let max = CAPTURE_SAMPLE_RATE as usize * LIVE_BUFFER_MAX_SECS;
        ingest(&dev.buffer, &dev.recording, &dev.captured, max, samples);
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    #[test]
    fn compute_append_room_for_full_chunk() {
        assert_eq!(compute_append(100, 50, 1000), 50);
    }

    #[test]
    fn resolve_device_ordinal_picks_in_range_index() {
        assert_eq!(resolve_device_ordinal(0, 3), Some(0));
        assert_eq!(resolve_device_ordinal(2, 3), Some(2));
    }

    #[test]
    fn resolve_device_ordinal_falls_back_when_out_of_range() {
        // Index equal to or beyond the device count -> None (use default).
        assert_eq!(resolve_device_ordinal(3, 3), None);
        assert_eq!(resolve_device_ordinal(9, 3), None);
    }

    #[test]
    fn resolve_device_ordinal_falls_back_when_no_devices() {
        assert_eq!(resolve_device_ordinal(0, 0), None);
    }

    #[test]
    fn compute_append_chunk_exactly_fills_buffer() {
        assert_eq!(compute_append(900, 100, 1000), 100);
    }

    #[test]
    fn compute_append_chunk_overflows_gets_clamped() {
        assert_eq!(compute_append(950, 100, 1000), 50, "only 50 of the 100 fit");
    }

    #[test]
    fn compute_append_already_full_takes_nothing() {
        assert_eq!(
            compute_append(1000, 50, 1000),
            0,
            "at capacity, take nothing"
        );
    }

    #[test]
    fn compute_append_past_full_takes_nothing() {
        // Shouldn't happen in practice but should not panic or wrap.
        assert_eq!(compute_append(2000, 50, 1000), 0);
    }

    #[test]
    fn compute_append_empty_buffer_empty_chunk() {
        assert_eq!(compute_append(0, 0, 1000), 0);
    }

    #[test]
    fn compute_append_empty_buffer_full_chunk() {
        assert_eq!(compute_append(0, 1000, 1000), 1000);
    }

    #[test]
    fn compute_append_realistic_audio_frame() {
        // 48 kHz, the live buffer's bound, 10 ms frame = 480 samples
        let max = CAPTURE_SAMPLE_RATE as usize * LIVE_BUFFER_MAX_SECS;
        let take = compute_append(max - 100, 480, max);
        assert_eq!(take, 100, "only 100 samples fit in the last slice");
    }

    #[test]
    fn classify_zero_samples_splits_on_hold_length() {
        // The historic bug: 0 samples logged as "Recording too short (0.00s)",
        // hiding a dead stream behind a user-timing message (TASK-VTT121).
        assert_eq!(
            classify_capture(0, 0, 3.0, CAPTURE_SAMPLE_RATE),
            CaptureClass::Empty
        );

        // A tap can end before cpal's first callback fires, so zero samples
        // over a hold shorter than MIN_DURATION_SECS is a quick user, not a
        // dead microphone. Reporting it as a dead device put a sticky error
        // icon in the tray and re-opened the stream for nothing — the symptom
        // that surfaced once releases stopped being discarded (TASK-VTT148).
        assert_eq!(
            classify_capture(0, 0, 0.02, CAPTURE_SAMPLE_RATE),
            CaptureClass::TooShort,
            "a hasty tap must not be reported as a dead microphone"
        );
        // At and beyond the threshold a silent stream is still a real fault.
        assert_eq!(
            classify_capture(0, 0, MIN_DURATION_SECS, CAPTURE_SAMPLE_RATE),
            CaptureClass::Empty
        );
        assert_eq!(
            classify_capture(0, 0, 10.0, CAPTURE_SAMPLE_RATE),
            CaptureClass::Empty
        );
    }

    #[test]
    fn classify_one_sample_is_too_short_not_empty() {
        // The boundary: any real samples below MIN_DURATION_SECS are the
        // user's timing, not a device failure.
        assert_eq!(
            classify_capture(1, 32767, 0.01, CAPTURE_SAMPLE_RATE),
            CaptureClass::TooShort
        );
    }

    #[test]
    fn classify_just_under_min_duration_is_too_short() {
        let just_under = (CAPTURE_SAMPLE_RATE as f32 * MIN_DURATION_SECS) as usize - 1;
        assert_eq!(
            classify_capture(just_under, 32767, 0.49, CAPTURE_SAMPLE_RATE),
            CaptureClass::TooShort
        );
    }

    #[test]
    fn classify_long_enough_but_quiet_is_too_quiet() {
        let one_second = CAPTURE_SAMPLE_RATE as usize;
        assert_eq!(
            classify_capture(one_second, MIN_AMPLITUDE - 1, 1.0, CAPTURE_SAMPLE_RATE),
            CaptureClass::TooQuiet
        );
    }

    #[test]
    fn classify_normal_recording() {
        let one_second = CAPTURE_SAMPLE_RATE as usize;
        assert_eq!(
            classify_capture(one_second, MIN_AMPLITUDE, 1.0, CAPTURE_SAMPLE_RATE),
            CaptureClass::Normal
        );
    }

    #[test]
    fn classify_a_long_recording_is_normal() {
        // There is no length limit to classify any more: a long hold is cut
        // into segments instead (TASK-VTT195).
        let ten_minutes = CAPTURE_SAMPLE_RATE as usize * 600;
        assert_eq!(
            classify_capture(ten_minutes, 20_000, 600.0, CAPTURE_SAMPLE_RATE),
            CaptureClass::Normal
        );
    }

    /// Play one hold as the app does: the callback delivers `audio` in 10 ms
    /// frames, the segment tick runs every second, then the key comes up.
    fn play_hold(dev: &Audio, audio: &[f32]) -> Vec<RecordingResult> {
        dev.start_recording();
        let mut out = Vec::new();
        for (i, frame) in audio.chunks(480).enumerate() {
            feed(dev, frame);
            if (i + 1) % 100 == 0 {
                out.extend(dev.take_segment());
            }
        }
        out.extend(dev.stop_recording());
        out
    }

    /// Each result's segment position and audio at the capture rate, with
    /// its debug wav removed.
    fn parts(results: Vec<RecordingResult>) -> Vec<(Part, Vec<f32>)> {
        results
            .into_iter()
            .map(|r| match r {
                RecordingResult::Audio {
                    path, native, part, ..
                } => {
                    std::fs::remove_file(&path).ok();
                    (part, native.expect("archiving is on").samples)
                }
                _ => panic!("expected a segment of audio"),
            })
            .collect()
    }

    #[test]
    fn a_long_hold_hands_on_every_sample_in_order() {
        // Half a minute at a 4 s threshold, released mid-phrase: several
        // cuts, each segment numbered in order, only the release's marked
        // last, and the segments back to back are the capture, sample for
        // sample.
        let dev = deviceless(4.0);
        let audio = phrases(31);
        let parts = parts(play_hold(&dev, &audio));
        assert!(parts.len() >= 5, "{} segments", parts.len());
        for (i, (part, _)) in parts.iter().enumerate() {
            assert_eq!(part.index as usize, i, "segments out of order");
            assert_eq!(part.last, i + 1 == parts.len(), "only the release is last");
        }
        let joined: Vec<f32> = parts.into_iter().flat_map(|(_, s)| s).collect();
        assert_eq!(joined.len(), audio.len(), "samples lost or added at a cut");
        assert!(joined == audio, "a sample changed or moved across a cut");
    }

    #[test]
    fn a_short_hold_is_one_whole_segment() {
        let dev = deviceless(4.0);
        let audio = phrases(3);
        let parts = parts(play_hold(&dev, &audio));
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].0, Part::WHOLE);
        assert!(parts[0].1 == audio);
    }

    #[test]
    fn silence_after_the_last_cut_still_delivers_the_earlier_segments() {
        let dev = deviceless(4.0);
        let mut audio = phrases(6);
        audio.extend(vec![0.0; CAPTURE_SAMPLE_RATE as usize]);
        let results = play_hold(&dev, &audio);
        assert_eq!(results.len(), 2, "one cut, then the release");
        assert!(matches!(
            &results[0],
            RecordingResult::Audio { part, .. } if *part == Part { index: 0, last: false }
        ));
        assert!(matches!(results[1], RecordingResult::NoFinalSegment));
        parts(results.into_iter().take(1).collect());
    }

    #[test]
    fn the_tick_never_drops_a_frame_the_callback_delivers() {
        // The callback and the tick contend for the buffer every second of a
        // hold. Every frame must survive the contention, in order.
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let recording = Arc::new(AtomicBool::new(true));
        let captured = Arc::new(AtomicUsize::new(0));
        let done = Arc::new(AtomicBool::new(false));
        let (frames, frame_len) = (20_000usize, 480usize);
        let producer = {
            let (b, r, c, d) = (
                buffer.clone(),
                recording.clone(),
                captured.clone(),
                done.clone(),
            );
            std::thread::spawn(move || {
                for f in 0..frames {
                    let data: Vec<f32> =
                        (0..frame_len).map(|i| (f * frame_len + i) as f32).collect();
                    ingest(&b, &r, &c, usize::MAX, &data);
                }
                d.store(true, Ordering::SeqCst);
            })
        };
        let mut drained = Vec::new();
        for _ in 0..100_000_000 {
            if done.load(Ordering::SeqCst) {
                break;
            }
            drained.extend(swap_out(&buffer, 0));
        }
        producer.join().expect("producer thread");
        drained.extend(swap_out(&buffer, 0));
        assert_eq!(captured.load(Ordering::SeqCst), frames * frame_len);
        assert_eq!(drained.len(), frames * frame_len, "frames dropped");
        assert!(
            drained.iter().enumerate().all(|(i, &s)| s == i as f32),
            "frames reordered"
        );
    }

    #[test]
    fn downmix_mono_is_passthrough() {
        let s = vec![0.1, -0.2, 0.3];
        assert_eq!(downmix_to_mono(&s, 1), s);
        assert_eq!(downmix_to_mono(&s, 0), s, "0 channels treated as mono");
    }

    #[test]
    fn downmix_stereo_averages_frame_pairs() {
        // Interleaved L,R,L,R: (1.0,-1.0)->0.0  (0.5,0.5)->0.5
        let stereo = vec![1.0, -1.0, 0.5, 0.5];
        assert_eq!(downmix_to_mono(&stereo, 2), vec![0.0, 0.5]);
    }

    #[test]
    fn resample_same_rate_is_passthrough() {
        let s = vec![0.0, 0.5, -0.5, 1.0];
        assert_eq!(resample_to_whisper(&s, WHISPER_SAMPLE_RATE), s);
        assert!(resample_to_whisper(&[], 48_000).is_empty());
    }

    #[test]
    fn whisper_input_is_16k_from_a_48k_capture() {
        // The invariant TASK-VTT150 rests on: capture moved to 48 kHz, and the
        // samples handed to Whisper are still 16 kHz, so transcription sees
        // exactly what it saw before. Three seconds in, three seconds out.
        let three_seconds: Vec<f32> = (0..(CAPTURE_SAMPLE_RATE as usize * 3))
            .map(|i| (i as f32 / 400.0).sin() * 0.4)
            .collect();
        let out = resample_to_whisper(&three_seconds, CAPTURE_SAMPLE_RATE);
        let expected = WHISPER_SAMPLE_RATE as usize * 3;
        assert!(
            (out.len() as i64 - expected as i64).abs() <= 1,
            "expected ~{expected} samples at 16 kHz, got {}",
            out.len()
        );
        assert!(out.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn whisper_resample_is_a_passthrough_for_a_16k_device() {
        // A device that only offers 16 kHz still works: the fallback captures
        // at its native rate and the resample becomes a copy, not a rescale.
        let s: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();
        assert_eq!(resample_to_whisper(&s, WHISPER_SAMPLE_RATE).len(), s.len());
    }

    #[test]
    fn resample_downsamples_48k_to_16k_thirds_the_length() {
        // 48 kHz -> 16 kHz is a 1:3 decimation; 300 in -> ~100 out.
        let input: Vec<f32> = (0..300).map(|i| (i as f32 * 0.01).sin()).collect();
        let out = resample_to_whisper(&input, 48_000);
        assert!(
            (out.len() as i32 - 100).abs() <= 1,
            "expected ~100 samples, got {}",
            out.len()
        );
        // Endpoints are preserved (no NaN, bounded).
        assert!((out[0] - input[0]).abs() < 1e-6);
        assert!(out.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn resample_upsamples_8k_to_16k_doubles_the_length() {
        let input: Vec<f32> = (0..50).map(|i| i as f32 / 50.0).collect();
        let out = resample_to_whisper(&input, 8_000);
        assert!(
            (out.len() as i32 - 100).abs() <= 1,
            "expected ~100 samples, got {}",
            out.len()
        );
        assert!(out.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn write_wav_round_trips_samples_through_hound() {
        // Synthesize a 100-sample sine-ish signal (amplitude 0.5 at peaks).
        let input: Vec<f32> = (0..100)
            .map(|i| (i as f32 / 100.0 * std::f32::consts::TAU * 5.0).sin() * 0.5)
            .collect();

        let path = write_wav(&input, WHISPER_SAMPLE_RATE).expect("write should succeed");
        assert!(path.exists(), "wav file should be on disk after write");

        // Read it back via hound (same crate used to write it).
        let mut reader = hound::WavReader::open(&path).expect("readable wav");
        let spec = reader.spec();
        assert_eq!(spec.sample_rate, WHISPER_SAMPLE_RATE);
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.bits_per_sample, 16);

        let decoded: Vec<f32> = reader
            .samples::<i16>()
            .map(|s| s.expect("readable sample") as f32 / 32767.0)
            .collect();

        assert_eq!(decoded.len(), input.len(), "sample count preserved");
        for (i, (&a, &b)) in input.iter().zip(decoded.iter()).enumerate() {
            // i16 quantization loses ~1/65536 of amplitude range — 0.0001 is a
            // comfortable upper bound that catches byte-order or scaling bugs
            // without being sensitive to the quantization noise floor.
            assert!(
                (a - b).abs() < 0.0001,
                "sample {} diverged: input={} decoded={}",
                i,
                a,
                b
            );
        }

        // Cleanup — don't leak /tmp WAVs from the test run.
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn write_wav_clamps_out_of_range_samples() {
        // Whisper can technically produce samples outside [-1, 1] if upstream
        // audio is normalised elsewhere. write_wav should clamp instead of
        // wrapping/panicking during the i16 cast.
        let input = vec![2.0_f32, -2.0, 0.5, -0.5, 1.5, -1.5];
        let path = write_wav(&input, WHISPER_SAMPLE_RATE).expect("write should succeed");

        let mut reader = hound::WavReader::open(&path).expect("readable wav");
        let decoded: Vec<i16> = reader
            .samples::<i16>()
            .map(|s| s.expect("readable sample"))
            .collect();

        assert_eq!(decoded.len(), input.len());
        assert_eq!(decoded[0], 32767, "2.0 clamps to +max");
        assert_eq!(decoded[1], -32767, "-2.0 clamps to -max");
        assert_eq!(decoded[4], 32767, "1.5 clamps to +max");
        assert_eq!(decoded[5], -32767, "-1.5 clamps to -max");
        // 0.5 / -0.5 round-trip near their expected i16 values.
        assert!((decoded[2] - 16383).abs() <= 1);
        assert!((decoded[3] + 16383).abs() <= 1);

        std::fs::remove_file(&path).ok();
    }
}
