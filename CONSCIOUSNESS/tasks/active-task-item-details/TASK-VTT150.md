# TASK-VTT150: Archive dictation as training-grade audio

## Context
Emmanuel dictates into this tool every day. Each recording is a paired audio/text
sample — exactly what the voice-clone project (`~/projects/auxiliary/epc-voice`,
TASK-EV034) needs hours of. Today both halves are thrown away: capture runs at
16 kHz, and the archive is a 20-file debug ring with no transcripts.

This task makes daily dictation training-grade, opt-in and off by default.

Mirrors TASK-EV034 (Extend voice-to-text to archive training-grade audio) in
epc-voice, which carries the full decision register and pre-mortem. Ownership was
handed to this seat by the epc-voice session on 2026-09-03; that session's
settings work is preserved at commit 18479b1.

## Measured starting state, 2026-09-03
- `src/audio.rs:10` — `const SAMPLE_RATE: u32 = 16000;` is both the capture rate and
  the Whisper input rate.
- `src/audio.rs:623` — `resample_to_16k(input, in_rate)` exists, used when a device
  cannot open 16 kHz directly.
- `src/main.rs:888` — `save_and_cleanup` copies the wav to
  `~/.local/share/voice-to-text/recordings/` then calls `prune_recordings(dir, 20)`.
- `src/whisper.rs` — `decode_wav_rejects_non_16khz` test asserts Whisper input stays
  16 kHz; the resample boundary must keep that true.
- Baseline: `cargo test --workspace` → 135 passed, 0 failed, 1 ignored.

## Acceptance Criteria
1. `cargo test --workspace` passes with at least three new tests: Whisper input is
   16 kHz from a 48 kHz capture; archiving off writes nothing new; archiving on
   writes wav plus sidecar
2. Five existing recordings transcribe to identical text before and after (outputs
   pasted on this card)
3. `ffprobe` on a newly archived file shows `sample_rate=48000`, `channels=1`
   (pasted on this card)
4. The sidecar json carries id, recorded_at, duration_s, sample_rate, text, model,
   language and app for that same recording
5. With the three settings absent, behaviour is byte-identical to today:
   `recordings/` still capped at 20, nothing written to an archive
6. `README.md` states what is recorded, where it is stored, how to disable it and
   how to delete it
7. Emmanuel has read that section and enabled archiving himself; this card records
   the date

## Technical Approach
1. Move the resample: capture at 48 kHz, call `resample_to_16k(&samples, 48000)`
   immediately before the Whisper call, so transcription input is unchanged.
2. Settings keys `archive_recordings`, `archive_dir`, `archive_max_files` — landed
   at 18479b1, defaults preserve today's behaviour when absent.
3. Archive write: `<archive_dir>/<YYYY-MM-DD>/vtt_<id>.wav` at 48 kHz 16-bit mono,
   plus `vtt_<id>.json` sidecar.
4. Cap the archive oldest-first, counting across dated directories.
5. README privacy section; CHANGELOG entry through the sanctioned flow.

## Test Strategy
Unit tests on the pure logic (resample boundary, archive path construction, cap
selection). Manual end-to-end: build a local binary, dictate, confirm a 48 kHz wav
and its sidecar appear and the typed text is unchanged.

## Files
- Modify: `src/audio.rs`, `src/main.rs`, `README.md`, `CHANGELOG.md`
- Already modified: `src/settings.rs` (18479b1)

## Out of Scope
- The transcription model, the hotkey, the typing path
- The existing 20-file `recordings/` debug ring (untouched — re-transcribe-last
  must keep working)
- Importing anything into epc-voice (TASK-EV035)
- Uploading archived audio anywhere
