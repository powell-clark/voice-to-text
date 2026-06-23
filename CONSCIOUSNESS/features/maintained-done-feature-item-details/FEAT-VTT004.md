---
id: FEAT-VTT004
status: maintained
kano: must-have
---

# FEAT-VTT004: Linux system tray with GTK3 and AppIndicator

## Description
On Linux, VTT displays a system tray icon using GTK3 and the `libappindicator3` library. The tray provides status display (Recording / Transcribing / Ready / Loading model), a model selection submenu, a logs submenu, and settings/quit actions. This is the primary user interface on Linux.

## Acceptance Criteria
- [x] Tray icon appears in the system tray on Ubuntu 24.04 (GNOME + AppIndicator extension) after `systemctl --user start vtt` — verified in daily use
- [x] Status text updates correctly through the lifecycle: Ready → Recording... → Transcribing... → Ready — verified
- [x] Model submenu shows available models with the current model checked — verified
- [x] Logs submenu shows the last N transcription log entries on hover — verified in v2.0.5 (fixed in TASK-VTT055)
- [x] Quit action stops the service cleanly — verified
- [x] No GTK warnings on launch (`G_MESSAGES_DEBUG=all`) under normal use — verify after fresh install

## Linked Tasks
- TASK-VTT004, TASK-VTT019

## Parent Story
- STORY-VTT001
