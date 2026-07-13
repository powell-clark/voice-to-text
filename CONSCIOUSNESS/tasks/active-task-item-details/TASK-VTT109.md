# TASK-VTT109: Windows autostart on by default — first-run enable + tray off-switch

## Context

Reported by Emmanuel (2026-06-26): "VTT doesn't auto-start on Windows." Verified
on Emmanuel's own machine — v2.3.7 installed, but `HKCU\...\Run\VoiceToText` had
no entry, so the app does not launch at login.

Root cause is not a bug: autostart (TASK-VTT094) shipped **opt-in, default off** —
the user must tick the tray "Start at login" item. Two compounding problems:
1. The expectation is that a voice tool is just *there* after a reboot.
2. The tray icon hides in the Windows 11 overflow flyout, so the toggle is hard
   to discover in the first place.

Decision (Emmanuel): most pro = on by default, with the tray menu item as the
off-switch. An installer-dialog checkbox was considered but rejected for this
pass — it needs custom WiX UI authoring and the WiX Toolset isn't available to
test locally, and a broken `.wxs` would fail the Windows `.msi` job and re-block
the release. Doing it in app code is testable and cannot break the installer.

## Approach

- Add a persisted `autostart_initialized` marker to `Settings` (settings.conf).
- On first launch on Windows (marker false), call `autostart::enable()` to write
  the HKCU Run entry, set the marker, and save — so the default applies exactly
  once, even across upgrades. Existing installs pick it up on next launch.
- The tray "Start at login" toggle remains the sole ongoing control; once the
  user opts out, the marker stays set and we never re-enable behind their back.

## Acceptance criteria

- [x] `Settings.autostart_initialized` persists across save/load (unit test)
- [x] First Windows launch enables start-at-login when the marker is unset
- [x] Marker is set only after a successful enable (transient failure retries)
- [x] Tray "Start at login" toggle still turns it off, and the off-state sticks
- [deferred] Verified on Windows: fresh launch writes HKCU Run; untick removes it; relaunch does not re-add
      — needs Windows hardware; not verifiable on the Linux dev box (no Windows
      target installed for cross-compile; CI windows-latest `.msi` job covers
      compilation). Emmanuel to confirm the HKCU Run round-trip on his next
      Windows launch. Mirrors the macOS deferral on TASK-VTT114 / REVIEW-VTT121.

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Feature: FEAT-VTT030 (autostart parity, FEAT-VTT015)
- Builds on: TASK-VTT094 (autostart module + tray toggle)
