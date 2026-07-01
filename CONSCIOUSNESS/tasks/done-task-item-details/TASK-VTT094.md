# TASK-VTT094: Start on login / autostart — Windows (and platform parity)

## Context

Requested by Emmanuel (2026-06-26): "don't know if this thing will start on
launch" — wants VTT to start automatically at login, with parity across platforms.

## Approach

- **Windows**: register under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
  (per-user, no admin) or drop a shortcut in the Startup folder. Expose a tray
  toggle "Start at login" that writes/removes the entry.
- **Linux**: an XDG autostart `.desktop` in `~/.config/autostart/` (the systemd
  `vtt.service` in packaging/linux is a different model — reconcile).
- **macOS**: a LaunchAgent plist in `~/Library/LaunchAgents/`.

A single cross-platform `autostart` module with per-OS impls, driven by one tray
toggle, keeps behaviour identical everywhere.

## Acceptance criteria

- [ ] Tray menu has a "Start at login" toggle reflecting current state
- [ ] Enabling it launches VTT on next login on each platform
- [ ] Disabling it removes the registration cleanly
- [ ] Default off until the user opts in

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Parity: row 13 in CONSCIOUSNESS/artifacts/feature-parity-matrix.md
