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

## Evidence so far (2026-09-05, vtt-c52f564e)

Log-based check of criterion 1 across the 8 days of local logs currently on disk
(`~/.local/share/voice-to-text/vtt-2026-08-29.log` through `vtt-2026-09-05.log`):

```
grep -h "fallback" ~/.local/share/voice-to-text/vtt-*.log | wc -l   -> 0
grep -h "Typing completed" ~/.local/share/voice-to-text/vtt-*.log | wc -l -> 3262
```

Zero `Typing fallback: N chars via clipboard paste` lines and 3,262 successful
`Typing completed` events across the window, with no `[TASK-VTT06X` regression
task filed since. This is well past the card's original v2.0.5 window (tagged
2026-04-20) — the installed version is now v2.4.0 (tag date 2026-09-05), several
releases past the v2.3.7 the title already flags as superseded.

**Not verifiable from logs, still needs the operator's own observation:**
criterion 2 (specific app coverage — Claude Code TUI, Firefox, Slack/Discord,
GNOME terminal, plain text editor — the log has byte counts, not which app or
which characters), and criterion 3 (tray Logs submenu populating on hover — no
corresponding log line exists to check).

A prior session today (vtt-690a6246) re-claimed this task four times in ~35
minutes with no new action to take and correctly wrote `STATUS: holding`
(`reason: status-holding`, `supervisor_hint: operator-resume`) rather than
spin further — this task is fundamentally gated on Emmanuel's own multi-day
usage and tray observation, not on further agent tool calls. Left in_progress
rather than re-claimed this session; recommend closing on Emmanuel's own
say-so once he confirms criteria 2 and 3, given criterion 1's evidence is now
strong and the version window has long since passed.
