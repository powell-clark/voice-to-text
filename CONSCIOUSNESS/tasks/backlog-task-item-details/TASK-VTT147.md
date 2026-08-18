# TASK-VTT147: Warn when the hotkey is an ordinary typing key

## Context

This machine's settings.conf carries hotkey=65, the space bar, so every space keypress opens a recording; most are discarded as too short but it makes any press/release race far more likely and keeps the mic cycling constantly. The settings dialog and startup log should flag a hotkey that maps to a printable character, and the tray should surface which key is bound. Found while diagnosing TASK-VTT146.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT015
- Directive: DIRECT-VTT002
