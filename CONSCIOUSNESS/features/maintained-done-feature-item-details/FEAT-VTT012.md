---
id: FEAT-VTT012
status: maintained
kano: must-have
---

# FEAT-VTT012: Clipboard paste via xclip subprocess

## Description
Transcribed text is pushed to the X11 clipboard using an `xclip` subprocess call rather than the broken `XSetSelectionOwner` API. This fixed crashes in browsers and Electron apps (Claude Code TUI, VS Code, Chrome) that occurred when VTT tried to own the X11 selection directly.

## Acceptance Criteria
- [x] **AC-1** — `xclip -selection clipboard` subprocess is called to write transcription to clipboard — verified in v2.0.0 source (`src/main.rs`)
- [x] **AC-2** — No `XSetSelectionOwner` call remains in the codebase — verified, removed in TASK-VTT012
- [x] **AC-3** — ctrl+v pastes transcribed text correctly in Firefox, Chrome, Claude Code TUI, and terminal — verified in v2.0.0 daily use
- [x] **AC-4** — VTT process does not crash on repeated clipboard writes across 50+ transcriptions — verified in v2.0.0 daily use
- [x] **AC-5** — `xclip` is listed in `debian/control` Depends so it is guaranteed present on install — verify via `dpkg -p voice-to-text`

## Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)
Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 4 — clipboard paste fallback).

**🐧 Linux — ✅ works**
- [x] Transcription written to the clipboard via `xclip`/`arboard`; `xclip` in `debian/control` Depends — `src/typing.rs:187-252`, `debian/control:36` (TASK-VTT110)
- [x] Manual Ctrl+V pastes correctly; no crash across 50+ writes

**🍎 macOS — 🟡 partial (TASK-VTT114)**
- [x] Transcription reaches the clipboard via `arboard` (manual Cmd+V works) — `src/typing.rs:187-216`
- [ ] Auto-paste keystroke uses **Cmd+V** on macOS — currently sends Ctrl+V (no-op); tracked as TASK-VTT114

**🪟 Windows — ✅ works**
- [x] Transcription written to the clipboard via `arboard` for manual Ctrl+V (no auto-paste, parity with the Linux fallback) — `src/typing.rs:99-103` (TASK-VTT099)

## Linked Tasks
- TASK-VTT012

## Parent Story
- STORY-VTT006
