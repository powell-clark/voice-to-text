# TASK-VTT062: Wire selected_device_index through to audio::Audio::new()

## Context
`Settings.selected_device_index: i32` has existed since v1.x. It's parsed
from `settings.conf` (`device=N` key), saved back, and exposed as a public
struct field. But `audio::Audio::new()` currently ignores it — always uses
`host.default_input_device()`.

Result: users with multiple microphones (headset + laptop mic + USB mic)
who want to pick one for VTT must change their system-wide PulseAudio
default via `pactl set-default-source ...`. They can't pick a VTT-specific
mic.

This is a p2 feature because the v2.0.5 audio error message now helpfully
lists available devices and points at the `pactl` fix, so users aren't
totally stuck. But a proper tray-menu "Microphone:" submenu would be
world-class UX and closer to paid alternatives (Otter, Dragon).

## Acceptance Criteria
1. `audio::Audio::new()` accepts or reads the `selected_device_index`
   and picks the matching cpal input device if set (>= 0), falling back
   to default if -1 or out of range.
2. Out-of-range / missing device: log a warning, fall back to default.
   Don't bail — user might have unplugged a USB mic.
3. Tray menu has a "Microphone" submenu showing:
   - "Default (<current default name>)" radio item
   - One radio item per available input device, labelled with the device
     description (not just the source name)
4. Selecting a new device:
   - Saves `selected_device_index` to settings.conf
   - Rebuilds the cpal stream in `Audio::new()` equivalent flow (can
     require daemon restart if cpal doesn't support hot-swap cleanly;
     acceptable but noted in the tray label: "Microphone: X (restart
     required)")
5. Device list refreshes on submenu open so hot-plugged devices appear
   without daemon restart.

## Technical Approach
1. Change `audio::Audio::new()` signature to
   `fn new(device_index: Option<usize>) -> Result<Self>` OR pass Settings.
2. Inside, if `device_index.is_some()`, iterate `host.input_devices()`
   and pick the one whose ordinal matches.
3. Log the selected device name at `Audio::new()` via `vtt_log!` so users
   can verify from logs which mic is actually open.
4. In `tray/linux.rs`, create a `build_microphone_menu(state)` that
   mirrors `build_logs_menu` — rebuilt on menu show, radio-group of all
   available devices, selection writes to settings and optionally
   triggers a daemon restart.

## Test Strategy
Manual on a machine with 2+ input devices:
- Laptop mic + USB headset: both visible in submenu, can switch.
- Unplug USB during session: VTT continues with laptop mic, not dead.
- Settings.conf round-trip: pick USB via tray, kill daemon, relaunch,
  USB is still selected.

## Files
- `src/audio.rs` — accept device_index parameter, lookup logic
- `src/tray/linux.rs` — add Microphone submenu
- `src/tray/portable.rs` — mirror for macOS/Windows parity
- `CHANGELOG.md` — note the feature in whatever release ships it

## Dependencies
- None new — cpal already exposes `host.input_devices()`.
