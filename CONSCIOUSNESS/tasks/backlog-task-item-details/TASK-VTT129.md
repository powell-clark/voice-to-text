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

## Measured finding, 2026-09-05 (blocks Linux implementation as scoped above)

Attempted to claim and build this task; picked up before writing any code
because the "labelled with the device description" criterion turned out to
be materially harder than the card assumed, on real hardware:

- This machine has a real USB mic (`/proc/asound/cards` card 1: "RØDE
  VideoMic GO II") plus an unused onboard HDA card (card 2: "Generic"),
  managed by PipeWire.
- `audio::list_input_device_names()` (cpal's raw ALSA enumeration) on this
  box returns 14 entries — `pipewire`, `pulse`, `default`, then eight
  `hw:`/`plughw:`/`sysdefault:`/`front:`/`surroundNN:`/`dsnoop:` variants,
  ALL for `CARD=Generic` (the unused onboard card). **The RØDE — the mic
  actually in use — does not appear at all**, almost certainly because
  PipeWire holds it in a way that fails cpal's own open-probe during
  enumeration.
- Net effect: a picker built directly from this list would bury the one
  real, in-use microphone and instead surface 8 confusing near-duplicate
  entries for a card with no mic plugged in, labelled with raw ALSA PCM
  strings (`hw:CARD=Generic,DEV=0`) rather than anything resembling
  "device description" — worse than shipping nothing.
- `get_default_mic_description()` (existing, pactl-based, used today for the
  passive "Microphone: <desc>" label) correctly resolves the RØDE via
  PipeWire's own view. So the friendly, correct data source for "what mic is
  this" is pactl, not cpal — but VTT's actual *capture* device selection
  (`selected_device_index` → `resolve_device_ordinal` → cpal ordinal) is
  keyed entirely off cpal's list. There is no reliable mapping between a
  pactl source name and a cpal ALSA ordinal on a PipeWire-managed system —
  they are different namespaces, and the one device a user would want to
  pick usually only exists in the pactl one.

This is a real architecture choice, not a wiring gap: either (a) Linux
selection moves to actually setting the PipeWire/PulseAudio default source
(`pactl set-default-source`) rather than a cpal ordinal — a system-wide side
effect outside VTT's own audio session, which changes behaviour for every
other app on the machine, or (b) the Linux picker is scoped down to
"Default" plus only genuine direct-hardware `hw:CARD=X` entries (one per
card, deduplicated, named via `/proc/asound/cards`) with a documented caveat
that PipeWire-routed USB devices are only reachable through "Default" — both
are legitimate, but it's an authorship-flow one-way-door call (an ADR), not
something to decide silently mid-task. Per DIRECT-VTT005 parity (criterion
4) the Linux and portable trays need to ship together, so this blocks the
whole task rather than just its Linux half — the portable side (Windows
WASAPI / macOS CoreAudio) is expected to be unaffected, since those cpal
backends report actual friendly device names rather than raw ALSA PCM
identifiers, but that is unverified on this Linux dev box.

## Dependencies
- TASK-VTT062 (wire-through) — done, provides the selection plumbing.
- Hardware: a machine with 2+ input devices for acceptance.
- An ADR deciding the Linux device-selection strategy (pactl default-source
  vs. hw:CARD-only cpal picker) before implementation resumes.
