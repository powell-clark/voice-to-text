# TASK-VTT139: Portable tray parity — settings and About dialogs

## Context

ADR-0005 accepted alternative (b): mirror parity gaps onto portable.rs as small independent tasks. This card covers two of the five documented gaps: (1) Customize Transcription Settings dialog (voice_prefix, 240-char initial_prompt with colour-coded counter, newline radios — linux.rs show_prompt_dialog lines 590-771 is the reference behaviour); (2) a real visible About window with selectable version/URL text (portable.rs MenuCmd::About currently only writes a log line). Logs submenu parity is already tracked separately as TASK-VTT098; hotkey dialog is the ADR-0005 spike task; live mic label (pactl-based) is explicitly descoped pending a portable equivalent design. Windows is the regression-testable target (operator dual-boots); macOS parked.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Features: FEAT-VTT030
