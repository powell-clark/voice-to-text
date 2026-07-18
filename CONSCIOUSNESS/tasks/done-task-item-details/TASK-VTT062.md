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
1. [x] `audio::Audio::new(device_index: Option<usize>)` picks the matching cpal
   input device when set, falling back to default when `None` or out of range.
   The `i32` sentinel is converted at the main.rs boundary via
   `usize::try_from(...).ok()` (`< 0` → `None`).
2. [x] Out-of-range / missing device: `resolve_device_ordinal` returns `None`,
   `open_capture_stream` logs a warning and falls back to default rather than
   bailing. `try_reopen` also honours the stored `device_index`, so recovery
   after a USB unplug does not silently revert to default. Unit-tested
   (`resolve_device_ordinal_*`); build/clippy/fmt/102 tests green on Linux.

Criteria 3–5 below are the tray "Microphone" submenu UX. **Deferred to
TASK-VTT129** — their acceptance is GUI + 2+-mic-hardware bound (see that
card's test strategy) and cannot be verified honestly in the headless sandbox.
The selection plumbing they build on is complete here.

3. [deferred → TASK-VTT129] Tray "Microphone" submenu (Default + per-device
   radio items labelled with the device description).
4. [deferred → TASK-VTT129] Selecting a device saves `selected_device_index`
   and rebuilds the stream (restart-required label acceptable).
5. [deferred → TASK-VTT129] Device list refreshes on submenu open so
   hot-plugged devices appear without a restart.

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
