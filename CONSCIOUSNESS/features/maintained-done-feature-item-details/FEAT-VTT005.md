---
id: FEAT-VTT005
status: maintained
kano: must-have
---

# FEAT-VTT005: Text injection into focused application (Linux via XTest)

## Description
After transcription completes, the text is typed into the currently focused application using the X11 XTest extension (`enigo` crate on Linux). The text appears at the cursor position as if the user typed it. Clipboard is also set so the user can paste if the injection does not land correctly.

**Note:** macOS text injection via Accessibility API is in the codebase skeleton but the macOS binary is not yet distributed. The maintained behaviour covered by this feature is the Linux XTest path.

## Acceptance Criteria
- [x] Transcribed text appears at the cursor position in the focused application without requiring a paste gesture — verified in daily use in Claude Code TUI, terminals, Firefox, Chrome, VS Code
- [x] Unicode characters including £, é, ñ, and emoji type correctly — verified in v2.0.5 (£/é typing fix in TASK-VTT055)
- [x] Clipboard is set to the transcription simultaneously so ctrl+v works as fallback — verified
- [x] Text injection does not produce duplicate characters under normal use — verified in daily use
- [x] Injection works in X11 and XWayland sessions — verified on Ubuntu 24.04 with GNOME/XWayland
- [x] `enigo` is used for keyboard simulation, not raw XTest calls — verified in source

## Linked Tasks
- TASK-VTT005, TASK-VTT018

## Parent Story
- STORY-VTT001
