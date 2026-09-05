# TASK-VTT170: Portable tray parity — Customize Transcription Settings dialog

## Context

Split from TASK-VTT139 (2026-09-05) — see that card's context for the full
reasoning. The Linux `show_settings_dialog` (`src/tray/linux.rs`) has grown to
10+ sections: hotkey capture, language, model, input device, voice prefix,
240-char initial prompt with a live character counter, corrections dictionary
(TASK-VTT144), newline behaviour, archive toggle, denoise toggle. muda/tray-icon
(the portable tray's toolkit) ships no dialogs or text-input widgets at all, so
mirroring this requires first deciding whether to add a real GUI toolkit
(`tao` + something, or per-OS native dialog templates) — that decision is
exactly what TASK-VTT138's spike exists to make, scoped narrowly to the hardest
single piece (interactive hotkey capture). Implementing this dialog before
that spike reports would mean picking the toolkit blind.

## Acceptance criteria

- [ ] _(to be scoped once TASK-VTT138 reports — the spike's chosen toolkit, or
      its "keep-split" verdict under ADR-0005, determines whether this dialog
      is buildable at all and with what)_

## Dependencies

- TASK-VTT138 (Spike hotkey dialog on portable tray) — hard blocker, decides
  the toolkit question this task needs answered first.
- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Features: FEAT-VTT030
