---
id: FEAT-VTT012
status: maintained
kano: must-have
---

# FEAT-VTT012: Clipboard paste via xclip subprocess

## Description
Transcribed text is pushed to the X11 clipboard using an `xclip` subprocess call rather than the broken `XSetSelectionOwner` API. This fixed crashes in browsers and Electron apps (Claude Code TUI, VS Code, Chrome) that occurred when VTT tried to own the X11 selection directly.

## Acceptance Criteria
- [x] `xclip -selection clipboard` subprocess is called to write transcription to clipboard — verified in v2.0.0 source (`src/main.rs`)
- [x] No `XSetSelectionOwner` call remains in the codebase — verified, removed in TASK-VTT012
- [x] ctrl+v pastes transcribed text correctly in Firefox, Chrome, Claude Code TUI, and terminal — verified in v2.0.0 daily use
- [x] VTT process does not crash on repeated clipboard writes across 50+ transcriptions — verified in v2.0.0 daily use
- [x] `xclip` is listed in `debian/control` Depends so it is guaranteed present on install — verify via `dpkg -p voice-to-text`

## Linked Tasks
- TASK-VTT012

## Parent Story
- STORY-VTT006
