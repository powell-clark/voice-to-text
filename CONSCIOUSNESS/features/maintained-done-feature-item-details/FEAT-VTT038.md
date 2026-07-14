---
id: FEAT-VTT038
status: maintained
kano: must-have
---

# FEAT-VTT038: Copy last transcription from the tray menu

## Kano
must-have — reclassified from delighter/p2 on operator instruction
(2026-07-14): a recovery net for lost dictation output is not optional
polish once shipped, it's a safety-critical escape hatch.

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
appears on both the Linux GTK tray (`src/tray/linux.rs`) and the portable
(Windows/macOS) tray (`src/tray/portable.rs`) per DIRECT-VTT005 parity.

## Acceptance criteria

- [x] After any successful transcription (normal or truncated), the tray item
      copies the exact final text (including corrections-dictionary output) to
      the clipboard — `LastTranscription` set post-`compose_final_text` in
      `main.rs`, read by both tray menu handlers
- [x] Safe no-op before the first transcription of a run — `None` branch logs
      and returns without touching the clipboard
- [x] Present and functional on Linux GTK tray and portable tray — menu item
      wired in both `src/tray/linux.rs` and `src/tray/portable.rs`
- [ ] Covered by a unit test for the last-transcription state handling —
      DEFERRED: no automated GTK/muda tray-interaction test harness exists;
      logic verified by code review only (REVIEW-VTT123). Revisit if a tray
      test harness is ever built.

## Status note

Implementing task (TASK-VTT123) shipped and closed 2026-07-13
(REVIEW-VTT123). This card was left sitting in the backlog with unchecked
boxes after the task closed — pure PGPS bookkeeping drift, not a product
gap. Corrected 2026-07-14 per operator instruction, moved to
maintained-done: tray menu items must be re-verified each release or they
can silently break, same class as the key-repeat filter (FEAT-VTT013) and
resident model (FEAT-VTT022).

## Stories

- STORY-VTT018 (regression tests / reliability) — nearest active story; recovery
  net for lost output

## Tasks

- TASK-VTT123 (Copy last transcription tray menu item)
