---
id: FEAT-VTT039
status: in_review
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
- [x] **AC-1** — A "Re-transcribe last recording" item appears on the Linux GTK tray
      (`src/tray/linux.rs`) and the portable tray (`src/tray/portable.rs`) —
      parity, DIRECT-VTT005.
- [x] **AC-2** — Activating it enqueues `WorkItem::RetranscribeLast`; the worker locates the
      newest `.wav` in the recordings dir, decodes it via
      `whisper::decode_wav_to_samples`, and re-transcribes + re-types.
- [x] **AC-3** — The re-transcribed WAV is NOT re-archived or pruned — the worker uses an
      empty `archive_path` so the `save_and_cleanup` guard (main.rs) is skipped;
      no self-copy / accidental delete of the source recording.
- [x] **AC-4** — Safe no-op when the recordings dir is empty/missing — worker logs and sets
      the "No recording to re-transcribe" status, no panic (unit-tested +
      worker `None` branch).
- [x] **AC-5** — Covered by unit tests for the `newest_wav` selector (newest-by-mtime,
      ignores non-wav, empty/missing dir → None). Tray click→signal wiring
      verified by code review — no headless GTK/muda test harness (same deferral
      basis as FEAT-VTT038).
- [x] **AC-6** — cargo fmt / clippy -D warnings / cargo test (111) green. ACTUAL PROOF: ran
      `--file` on the newest real archived recording — the exact
      `newest_wav → decode → transcribe` path RetranscribeLast uses — and got a
      correct transcript. The GUI click→re-type step is GUI-bound (deferred to
      manual verification, per FEAT-VTT038 basis).

## Stories
- STORY-VTT018 (regression tests / reliability) — recovery net for lost output.

## Tasks
- TASK-VTT132 (Re-transcribe last recording tray item)


## Verification, 2026-09-03

Marked never-verified until now. The capture-rate change in TASK-VTT150
(Archive dictation as training-grade audio) is the first thing that could have
broken this feature silently, so the chain it depends on was checked
end-to-end rather than assumed.

This feature reads the newest wav from the debug ring and re-runs Whisper on
it. That only works while three things hold:

1. The ring still fills. 20 wavs present, at the cap, newest
   `vtt_recording_oyb7wB.wav`.
2. Those wavs are still 16 kHz mono. `ffprobe` says `sample_rate=16000`,
   `channels=1` — despite capture having moved to 48 kHz, because
   `stop_recording` resamples before writing the debug wav specifically so this
   recovery net keeps working.
3. The decoder still accepts them. `whisper::decode_wav_to_samples` rejects
   anything other than `WHISPER_INPUT_RATE` (16 kHz), which the ring satisfies.

Had TASK-VTT150 written 48 kHz to `recordings/` — the obvious implementation —
this feature would have broken with a decode error on every use, and nothing
would have caught it until someone actually needed to recover lost dictation.
That was the design risk of the archive work and the reason the archive is a
separate path rather than a widened one.

What is still NOT verified is the tray click itself: that the menu item fires,
the worker picks up `WorkItem::RetranscribeLast`, and the text re-types into
the focused window. That needs a human at the tray, and it is the remaining gap
before this must-have feature can be approved.
