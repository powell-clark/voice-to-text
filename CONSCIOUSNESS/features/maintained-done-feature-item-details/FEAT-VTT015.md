---
id: FEAT-VTT015
status: done
kano: performance
---

# FEAT-VTT015: SystemD service inherits DISPLAY from user session

## Description
The `vtt.service` systemd unit is configured to run in the user session (not as a system service) and inherits the `DISPLAY` and `XAUTHORITY` environment variables so the GTK tray and X11 keyboard capture work correctly. This also makes the service forward-compatible with Wayland via XWayland.

## Acceptance Criteria
- [x] **AC-1** — `vtt.service` runs as a user service (`systemctl --user`) not a system service — verified in `packaging/linux/vtt.service`
- [x] **AC-2** — Service has access to the DISPLAY variable when started by systemd — verified in daily use on Ubuntu 24.04
- [x] **AC-3** — Tray icon appears correctly after `systemctl --user start vtt` — verified
- [x] **AC-4** — Hotkey capture works correctly when service is started via systemd — verified
- [x] **AC-5** — `journalctl --user -u vtt` shows no DISPLAY or X11 connection errors — verify after fresh install

## Linked Tasks
- (delivered as part of TASK-VTT019 / packaging work)

## Parent Story
- STORY-VTT006
