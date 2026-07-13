# TASK-VTT129: Tray "Microphone" submenu — device picker UX

## Context
Split out from TASK-VTT062. That task wired `selected_device_index` through to
`audio::Audio::new()` so a saved `device=N` is actually honoured (with default
fallback on out-of-range). What remains is the *tray UX* for choosing a device
without hand-editing `settings.conf` — criteria 3–5 of the original card.

Deferred because its acceptance is GUI + hardware bound: the test strategy
requires a machine with 2+ input devices (laptop mic + USB headset), hot-plug
during a session, and a settings round-trip across a daemon restart. None of
that is verifiable in the headless CI/dev sandbox, so it needs a real
multi-mic machine to land honestly.

## Acceptance Criteria
1. Tray has a "Microphone" submenu showing a "Default (<current default name>)"
   radio item plus one radio item per available input device, labelled with the
   device description (not just the source name).
2. Selecting a device saves `selected_device_index` to settings.conf and
   rebuilds the capture stream (a daemon restart is acceptable if cpal can't
   hot-swap cleanly — note it in the label: "Microphone: X (restart required)").
3. The device list refreshes on submenu open so hot-plugged devices appear
   without a restart.
4. Present on both the Linux GTK tray and the portable (Windows/macOS) tray
   (DIRECT-VTT005 parity).
5. Manual verification on 2+ input devices: switch mics; unplug USB mid-session
   and confirm VTT continues on the remaining mic; settings.conf round-trips
   across a daemon restart.

## Technical Approach
In `tray/linux.rs`, add a `build_microphone_menu(state)` mirroring
`build_logs_menu` — rebuilt on menu show, radio-group of all available devices,
selection writes settings and triggers the stream rebuild. Mirror on
`tray/portable.rs`. The lookup half (ordinal → cpal device, fallback) already
exists in `audio::resolve_device_ordinal` / `Audio::new`.

## Dependencies
- TASK-VTT062 (wire-through) — done, provides the selection plumbing.
- Hardware: a machine with 2+ input devices for acceptance.
