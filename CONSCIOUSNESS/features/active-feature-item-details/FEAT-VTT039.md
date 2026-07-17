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
- [x] A "Re-transcribe last recording" item appears on the Linux GTK tray
      (`src/tray/linux.rs`) and the portable tray (`src/tray/portable.rs`) —
      parity, DIRECT-VTT005.
- [x] Activating it enqueues `WorkItem::RetranscribeLast`; the worker locates the
      newest `.wav` in the recordings dir, decodes it via
      `whisper::decode_wav_to_samples`, and re-transcribes + re-types.
- [x] The re-transcribed WAV is NOT re-archived or pruned — the worker uses an
      empty `archive_path` so the `save_and_cleanup` guard (main.rs) is skipped;
      no self-copy / accidental delete of the source recording.
- [x] Safe no-op when the recordings dir is empty/missing — worker logs and sets
      the "No recording to re-transcribe" status, no panic (unit-tested +
      worker `None` branch).
- [x] Covered by unit tests for the `newest_wav` selector (newest-by-mtime,
      ignores non-wav, empty/missing dir → None). Tray click→signal wiring
      verified by code review — no headless GTK/muda test harness (same deferral
      basis as FEAT-VTT038).
- [x] cargo fmt / clippy -D warnings / cargo test (111) green. ACTUAL PROOF: ran
      `--file` on the newest real archived recording — the exact
      `newest_wav → decode → transcribe` path RetranscribeLast uses — and got a
      correct transcript. The GUI click→re-type step is GUI-bound (deferred to
      manual verification, per FEAT-VTT038 basis).

## Stories
- STORY-VTT018 (regression tests / reliability) — recovery net for lost output.

## Tasks
- TASK-VTT132 (Re-transcribe last recording tray item)
