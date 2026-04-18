# TASK-VTT028: Route raw f32 samples from audio.rs to worker without WAV round-trip

## Context
`audio.rs` captures raw `f32` samples at 16 kHz mono in `self.buffer` (already the native format whisper-rs expects). On recording stop, `stop_recording()` converts those samples to i16 PCM and writes a WAV to `/tmp`, returning a `PathBuf`. The worker then reads that WAV back, converts i16 to f32, and passes to the engine. This round-trip loses information (f32 → i16 → f32 quantisation), wastes I/O, and adds 10-50 ms of latency per transcription.

This task routes the raw f32 buffer directly to the worker via the mpsc channel, keeping the WAV write only for the debug recordings archive (which the user values for troubleshooting transcription failures).

## Acceptance Criteria
1. `WorkItem::Audio` changes from `Audio(PathBuf)` to `Audio { samples: Vec<f32>, archive_path: PathBuf }` — the archive path is still included so `save_and_cleanup` can move the WAV to the recordings archive after transcription
2. `Audio::stop_recording` returns the full `Vec<f32>` sample buffer alongside the `PathBuf` of the WAV; the WAV is still written for archive purposes but the worker consumes the in-memory samples directly
3. The worker no longer calls `load_audio_wav` on the hot path — it operates on the `&[f32]` already in the `WorkItem`
4. Transcription latency measured on a 5-second clip is at least 10 ms faster than the WAV-read path (verifiable via log timestamps)
5. The `~/.local/share/voice-to-text/recordings/` archive still contains each transcribed clip as a WAV file, unchanged
6. If the WAV write fails (disk full, permission), transcription still proceeds because samples are in memory; only the archive copy is lost with a warning log

## Technical Approach
In `audio.rs`:
- `RecordingResult::Audio(PathBuf)` → `RecordingResult::Audio { samples: Vec<f32>, path: PathBuf }` — same for `Truncated` and `MaxLength`
- `stop_recording` takes a snapshot of `self.buffer` (clone or `std::mem::take`) before converting to WAV; returns both

In `main.rs`:
- `WorkItem::Audio { samples, archive_path }` — worker reads from `samples` directly

In the hotkey callback (currently in `main.rs` `on key release`):
- Destructures `RecordingResult::Audio { samples, path }`, sends `WorkItem::Audio { samples, archive_path: path }`

## Test Strategy
Record five clips, verify each transcribes to identical text as the WAV-round-trip path (samples should be bit-identical going in, so transcription output must match). Delete the archive directory — transcription must still succeed for the next press.

## Files
- `src/audio.rs` (modify — `RecordingResult` variants)
- `src/main.rs` (modify — `WorkItem` enum, hotkey callback, worker)
