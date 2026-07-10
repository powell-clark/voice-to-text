---
id: FEAT-VTT038
status: backlog
kano: delighter
---

# FEAT-VTT038: Copy last transcription from the tray menu

## Kano
delighter (p2)

## Description

A "Copy last transcription" tray menu item that puts the most recent
transcription text on the clipboard. Recovery net for the case where typed
output is lost — wrong window focused, target app discarded the input, or the
typing path failed. Requested by Emmanuel after losing a dictation with no way
to recover the text (2026-07-09, session vtt-main-951bfc78): audio WAVs are
archived for debugging, but the *text* result is discarded the moment it is
typed.

Explicitly a safety net, not the fix for lost output itself — typing
reliability remains owned by the typing/injection features. Cross-platform:
must appear on both the Linux GTK tray and the portable (Windows/macOS) tray
per DIRECT-VTT005 parity.

## Acceptance criteria

- [ ] After any successful transcription (normal or truncated), the tray item
      copies the exact final text (including corrections-dictionary output) to
      the clipboard
- [ ] Safe no-op before the first transcription of a run
- [ ] Present and functional on Linux GTK tray and portable tray
- [ ] Covered by a unit test for the last-transcription state handling

## Stories

- STORY-VTT018 (regression tests / reliability) — nearest active story; recovery
  net for lost output

## Tasks

- TASK-VTT123 (Copy last transcription tray menu item)
