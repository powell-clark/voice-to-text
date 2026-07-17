---
id: FEAT-VTT039
status: in_progress
kano: must-have
---

# FEAT-VTT039: Re-transcribe last recording from the tray

## Kano
must-have — a recovery net for lost dictation output, same class as
FEAT-VTT038 (Copy last transcription). When the typed output is lost (wrong
window focused, target app discarded input, typing path failed), the newest
archived WAV is the durable artifact; re-running whisper on it and re-typing
recovers the text without the operator re-recording. Not optional polish once
shipped — a safety-critical escape hatch.

## Description
A "Re-transcribe last recording" tray menu item that finds the most recent
archived WAV (`~/.local/share/voice-to-text/recordings/`, newest survives
pruning), decodes it, re-runs transcription, and **re-types** the result.
Requested by Emmanuel (2026-07-17, session vtt-main-690a6246): "a button that
tries to transcribe the last recording if it fails, or just automatically does
it if it fails".

Re-type (not copy) is deliberate: it recovers output the same way the original
dictation lands, sidesteps the clipboard entirely (relevant now that CopyQ is
gone — see FEAT-VTT038 / TASK-VTT131), and matches "retry the output". Works
across process restarts because it reads from disk, not in-memory state.

Auto-retry on the transcription-worker `None` branch is intentionally out of
scope: whisper failures on identical samples are deterministic, so an automatic
re-run of the same audio would just fail again. The real failure mode is
lost typing / focus, which the manual button recovers. Cross-platform per
DIRECT-VTT005 parity: Linux GTK tray and portable (Windows/macOS) tray.

## Acceptance criteria
- [ ] A "Re-transcribe last recording" item appears on the Linux GTK tray and
      the portable tray (parity, DIRECT-VTT005).
- [ ] Activating it locates the newest `.wav` in the recordings dir, decodes it
      via `whisper::decode_wav_to_samples`, and enqueues it to the transcription
      worker for re-transcription + re-typing.
- [ ] The re-transcribed WAV is NOT re-archived or pruned (empty archive_path so
      the worker skips `save_and_cleanup`) — no self-copy / accidental delete of
      the source recording.
- [ ] Safe no-op with a log line when the recordings dir is empty (nothing
      recorded yet this install).
- [ ] Covered by a unit test for the "newest WAV in a directory" selector (pure,
      no GUI); tray wiring verified by code review (no headless GTK/muda test
      harness — same deferral basis as FEAT-VTT038).
- [ ] cargo fmt / clippy -D warnings / cargo test green; actual-proof re-type of
      a known recording on Linux.

## Stories
- STORY-VTT018 (regression tests / reliability) — recovery net for lost output.

## Tasks
- TASK-VTT132 (Re-transcribe last recording tray item)
