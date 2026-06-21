# TASK-VTT054: Settings toggle for CT2 daemon backend

## Acceptance Criteria
1. `settings.conf` has a `backend = [native|ct2]` field; default is `native`
2. When `ct2` is selected, the daemon is launched at startup if not already running
3. If the daemon crashes mid-session, the app falls back to whisper-rs without user intervention
4. The tray shows "Backend: CT2" or "Backend: Native" in the status area
