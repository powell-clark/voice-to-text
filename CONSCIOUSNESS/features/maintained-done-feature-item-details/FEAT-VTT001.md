---
id: FEAT-VTT001
status: maintained
kano: must-have
---

# FEAT-VTT001: Push-to-talk voice recording

## Description
The user holds a configurable hotkey (default: F4) to record audio. Recording starts on keypress and stops on release. Audio is captured at 16 kHz mono via the system's default microphone through the `cpal` audio library. The raw f32 samples are sent directly to the Whisper worker thread — no WAV round-trip in the hot path (WAV files are written only to the debug archive).

## Acceptance Criteria
- [x] **AC-1** — Pressing and holding the hotkey starts recording and the tray shows `Recording...` — verified in daily use on Ubuntu 24.04
- [x] **AC-2** — Releasing the hotkey stops recording and the tray transitions to `Transcribing...` — verified
- [x] **AC-3** — Audio is captured at 16 kHz mono — verified in `src/audio.rs` (SAMPLE_RATE = 16000, channels = 1)
- [x] **AC-4** — `cpal` default input device is used — verified in `src/audio.rs`
- [x] **AC-5** — Raw f32 samples are sent to the worker channel without WAV serialisation — verified in `src/main.rs` after TASK-VTT028
- [x] **AC-6** — Minimum recording quality filters (silence detection, noise floor) applied before sending — verify in `src/audio.rs`
- [x] **AC-7** — The hotkey is configurable via `settings.conf` `hotkey` field — verified

## Linked Tasks
- TASK-VTT001, TASK-VTT016

## Parent Story
- STORY-VTT001
