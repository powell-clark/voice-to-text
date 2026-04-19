# TASK-VTT056: Monitor v2.0.5 typing fix in daily use

## Context
The v2.0.5 fix for £/é typing was validated by reading the code and rebuilding — not by automated tests (those come later in this story). The only way to confirm the class-of-bug is gone is to dictate daily in the apps where the old bug manifested and confirm none of them silently drop the tail of a transcription.

## Acceptance Criteria
1. Over 3-5 days of normal dictation, zero transcriptions show the "typing stops at first non-ASCII char" failure mode
2. At least 10 transcriptions containing £, é, —, quote marks, or other non-ASCII chars have been dictated across Claude Code TUI, Firefox, Slack/Discord, a GNOME terminal, and a plain text editor
3. Tray Logs submenu populates on first hover in at least 5 separate tray opens across different days
4. If any regression is observed, open a new task (TASK-VTT06X) with the failing transcription text, the target app, and the log excerpt showing byte counts

## Technical Approach
Passive monitoring — this is user observation, not a scripted test. The pragmatic check is `grep "Typing" ~/.local/share/voice-to-text/vtt-*.log | grep -v "Typing completed"` after each day — a healthy log only shows `Typing N bytes` followed by `Typing completed (N chars typed directly)` pairs. Any `Typing fallback: N chars via clipboard paste` lines for plain Latin text signal that Key::Unicode isn't working as expected on this X11 setup and need investigating.

## Test Strategy
Self-reporting. If no issues observed after 5 days, mark this task done and stop monitoring.

## Files
- No file changes — observational task only
