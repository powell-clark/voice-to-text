# TASK-VTT092: Windows typing drops characters and reorders text

## Context

Reported by Emmanuel (2026-06-26) on Windows: transcription is correct but the
typed output is garbled. Example — transcript "Testing one two three…" typed as
`[oice] esting one, two, three…` with stray `VT` appended. Leading/upper-case
characters are dropped and re-inserted out of order.

## Root cause

`src/typing.rs` is Linux-first: it shells out to `xdotool` (not present on
Windows), logs "xdotool not found", then falls back to `enigo` char-by-char via
`Key::Unicode(c)`. On Windows that path drops some characters (notably the first
/ capitals); the dropped chars are collected and pasted via the clipboard at the
**end**, producing reordered output (`…three.VT`). The per-press `xdotool` probe
also adds a failed-spawn on every transcription.

## Fix (proposed)

On Windows, bypass the xdotool probe entirely and type reliably — either
`enigo.text()` for whole segments (single SendInput batch, no per-char race) or a
save/set/Ctrl+V/restore clipboard paste of the full text. Preserve the
newline_type (Shift+Return vs Return) handling. Needs on-hardware verification.

## Acceptance criteria

- [ ] Windows types the full transcript verbatim — no dropped leading/upper-case
      characters, no reordering, no stray trailing characters
- [ ] No per-transcription "xdotool not found" on Windows (skip the probe)
- [ ] newline handling (Shift+Return vs Return) preserved
- [ ] Linux xdotool path unchanged

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Found-by: real use of v2.3.0 on Windows
- Relates: FEAT-VTT005 (text injection)
