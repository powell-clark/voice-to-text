# TASK-VTT123: Copy last transcription tray menu item

## Context

Requested by Emmanuel (2026-07-09 session vtt-main-951bfc78): a transcription
was lost (typed into a window that discarded it / typing failed) and there was
no way to get it back. "Can we get a copy last message button? On the drop
down. Copy last transcription in case I miss it."

Emmanuel's own framing: it's a feature papering over a bug (typed output should
never be lost), but it is independently useful as a recovery net — transcripts
are already archived as WAVs, yet the *text* result is discarded after typing.

## Approach

1. Keep the most recent transcription text in shared state (e.g.
   `Arc<Mutex<String>>` updated by the transcription worker after each
   successful transcription, including truncated ones).
2. Add a "Copy last transcription" item to both trays (GTK + portable) that
   places that text on the clipboard (existing xclip path on Linux; arboard or
   the platform equivalent already used for clipboard fallback on Windows).
3. Grey out / no-op with a log line when no transcription has happened yet this
   run.

Implements FEAT-VTT038.

## Acceptance criteria

- [ ] After a successful transcription, tray menu "Copy last transcription"
      puts the exact typed text on the clipboard
- [ ] Works for `[Truncated]` max-length results too
- [ ] Selecting the item before any transcription this run is a safe no-op
- [ ] Present on both the Linux GTK tray and the portable (Windows/macOS) tray
- [ ] cargo test / clippy / fmt green

## Dependencies

- Feature: FEAT-VTT038 (Copy last transcription)
- Directive: DIRECT-VTT005 (cross-platform parity — item must land on both trays)
