# TASK-VTT099: Windows — set clipboard as Ctrl+V fallback after typing

Parity with Linux FEAT-VTT012 (clipboard set simultaneously with typing). The new
Windows enigo.text() path typed text but did not set the clipboard. Now sets it via
arboard after typing so Ctrl+V works as a fallback.

- [x] Clipboard holds the transcription after each type on Windows
- [ ] Verified: Ctrl+V pastes the last transcription on Windows
- Story: STORY-VTT013 · Directive: DIRECT-VTT004 · Parity §3
