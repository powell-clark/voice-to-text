# STORY-VTT020: Dictation archived at training quality

## User Story
As Emmanuel I want my ordinary daily dictation archived at training quality with its transcript so that my voice-clone corpus grows passively, without me sitting down to record.

## Why This Matters
The voice-clone project (`~/projects/auxiliary/epc-voice`, DIRECT-EV001) needs hours of paired
audio and text in Emmanuel's voice. Deliberate recording sessions are capped at two hours of his
time by FEAT-EV001 AC-8, which is nowhere near enough on its own. He already dictates into this
tool every day, and every one of those recordings is a paired audio/text sample — it is simply
being thrown away.

Two things stop today's recordings being usable as training data:

1. Capture runs at 16 kHz (`src/audio.rs:10`). Whisper wants 16 kHz; TTS training wants 24 kHz or
   better, and 48 kHz is the microphone's native rate. Upsampling 16 kHz audio does not recover
   what was never captured.
2. The archive is a 20-file debug ring buffer (`src/main.rs:906`) with no transcript sidecars, so
   the text half of each pair is lost the moment the next recording lands.

Both are cheap to fix. The constraint that makes it delicate is that this is a shipped public
product Emmanuel uses every day: a transcription regression breaks his dictation, and a product
that writes users' voices to disk must be opt-in, documented and deletable.

## Acceptance Criteria
1. Capture runs at 48 kHz; resampling to 16 kHz happens at the Whisper call site, so transcription
   input is unchanged
2. Five existing recordings transcribe to identical text before and after the change
3. Three settings keys (`archive_recordings`, `archive_dir`, `archive_max_files`), all absent by
   default, with absent behaviour byte-identical to today
4. With archiving on, each recording writes a 48 kHz 16-bit mono wav plus a JSON sidecar carrying
   id, recorded_at, duration_s, sample_rate, text, model, language and focused app
5. The archive is capped oldest-first across dated directories
6. `README.md` states plainly what is recorded, where it is stored, how to disable it and how to
   delete it
7. `cargo test --workspace` passes with new tests covering the resample boundary, archiving off
   and archiving on

## Scope
- **In scope:** capture rate, the resample boundary, the opt-in archive path and its sidecar, the
  archive cap, settings, privacy documentation, tests
- **Out of scope:** the transcription model, the hotkey, the typing path, the existing 20-file
  `recordings/` debug ring (untouched, so re-transcribe-last still works), importing anything into
  epc-voice, uploading audio anywhere

## Tasks
- (filed on the next commit)

## Cross-repo
Mirrors TASK-EV034 (Extend voice-to-text to archive training-grade audio) in `epc-voice`, which
carries the full specification, the decision register and the pre-mortem. Read that card first.
