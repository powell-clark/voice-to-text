# 10. Linux microphone device-selection strategy for the tray picker

Date: 2026-09-05

## Status

Proposed — pending operator sign-off. Filed per TASK-VTT129 (STORY-VTT016,
DIRECT-VTT005), which measured the blocking problem in detail before this
ADR existed; this file turns that finding into a decision request rather
than leaving it as an unresolved note on the task card.

## Context

TASK-VTT129 wants a tray "Microphone" submenu so a user can pick their input
device without hand-editing `settings.conf`. The plumbing already exists —
`selected_device_index` → `audio::resolve_device_ordinal` → `cpal`'s device
list — from TASK-VTT062. What's missing is the picker UI, and building it
surfaced a real architecture problem, not just a UI gap.

Measured on this machine (Ubuntu 24.04, PipeWire): `/proc/asound/cards`
shows two cards — a real USB mic (card 1, "RØDE VideoMic GO II", currently
in use) and an unused onboard HDA card (card 2, "Generic"). `cpal`'s raw
ALSA enumeration (`audio::list_input_device_names`, used today only for a
`--list-devices` diagnostic, not the picker) returns 14 entries — `pipewire`,
`pulse`, `default`, and eight `hw:`/`plughw:`/`sysdefault:`/`front:`/
`surroundNN:`/`dsnoop:` variants — every one of them for the unused
`CARD=Generic`. **The RØDE does not appear at all**, almost certainly
because PipeWire holds the device in a way that fails `cpal`'s own
open-probe during enumeration.

By contrast, `get_default_mic_description()` (existing, `pactl`-based,
already used today for the passive "Microphone: <desc>" tray label)
correctly resolves the RØDE via PipeWire's own view — friendly name,
correct device. The two enumeration paths live in genuinely different
namespaces: `pactl` sources vs. `cpal`/ALSA PCM identifiers. There is no
reliable mapping from one to the other on a PipeWire-managed system, and
the one device most users would actually want to pick tends to only exist
in the `pactl` one.

Building the picker directly from `cpal`'s list, as the original task card
assumed, would surface eight confusing near-duplicate entries for a card
with nothing plugged into it, labelled with raw ALSA strings
(`hw:CARD=Generic,DEV=0`), while burying or omitting the microphone
actually in use. That is worse than shipping nothing.

## Decision

**Proposed: adopt alternative (a) below** — pending explicit operator
sign-off, because it has a real, non-cosmetic side effect (see Consequences).
Not yet accepted; implementation does not resume until this Status changes.

## Considered Alternatives

### (a) Tray selection sets the PipeWire/PulseAudio default source (`pactl set-default-source`)

Enumerate devices via `pactl list sources` (the same call
`get_default_mic_description()` already makes) for the picker's labels.
Selecting an entry runs `pactl set-default-source <name>`. VTT's own
capture keeps using `cpal`'s existing "default" device — no change to
`selected_device_index`/`resolve_device_ordinal` at all — because `cpal`'s
default device on a PipeWire system already resolves through the ALSA
compatibility "default" alias, which itself tracks whatever PipeWire
considers the current default source. In effect, this reuses the capture
path that's already shipped and correct; only the enumeration/selection UI
changes.

Verified: `audio::Audio::new()` is called exactly once, at startup
(`src/main.rs:234`), and resolves `host.default_input_device()` at that
point — so a `pactl set-default-source` change while VTT is already running
needs a restart to take effect, same as the "(restart required)" case the
original task's AC-2 already anticipates for ordinal-based switching. Not a
new caveat this alternative introduces.

**Pros:**
- Correctly resolves PipeWire-routed USB devices with friendly names — the
  exact case measured above, and almost certainly the common case on any
  Ubuntu 24.04+ desktop, since PipeWire is the default sound server.
- Small implementation: enumeration code already exists
  (`get_default_mic_description`'s `pactl` call is the same shape as
  listing all sources); no new mapping layer between two device namespaces
  is needed because capture keeps using `cpal`'s "default" device
  unconditionally.
- Matches user intent directly — "use this mic" reads naturally as "this is
  now my system's input", which is what `pactl set-default-source` does.

**Cons / risks:**
- **System-wide side effect.** Changing the default source from VTT's tray
  changes the microphone every other application on the machine uses too
  (Zoom, Discord, browser calls, OBS, etc.) — this is the one-way-door
  property that makes this an ADR rather than a wiring task. A user who
  expects VTT-scoped device selection would be surprised.
- Depends on shelling out to `pactl` at selection time (same dependency
  class already accepted for the passive mic-label feature, not a new
  external tool).
- Does nothing for a non-PipeWire, non-PulseAudio Linux setup (rare on
  modern Ubuntu, but not zero) — `pactl` would simply fail, and the picker
  would need a graceful fallback (likely "Default" only).

### (b) Scope the Linux picker down to "Default" + genuine `hw:CARD=X` entries

Deduplicate `cpal`'s list to one entry per physical card via
`/proc/asound/cards` (skip the `plughw:`/`sysdefault:`/`front:`/
`surroundNN:`/`dsnoop:` variants), label each from the card name, and
document that PipeWire-routed USB devices are only reachable through
"Default".

**Pros:**
- No system-wide side effect — stays entirely within VTT's own capture
  session, changing only `selected_device_index`.
- No new external-tool dependency at selection time.

**Cons / risks:**
- **Does not solve the problem that motivated the feature.** On the exact
  hardware measured above — a USB mic on a PipeWire-managed Ubuntu
  desktop — the device a user would actually want to pick does not appear
  in this list at all. This alternative would ship a picker that works
  only for direct-hardware onboard/PCI cards, which is the minority case on
  a modern laptop with a USB/Bluetooth mic or headset.
- The "Default" entry still exists as an escape hatch, but that is exactly
  the pre-existing status quo (no picker) with extra menu clutter added on
  top.

### (c) Move Linux capture off `cpal`/ALSA onto PipeWire's native API

Use `libpipewire` directly for both capture and device listing, sidestepping
the ALSA-compatibility enumeration gap entirely.

**Pros:**
- Would fix the root cause (the enumeration gap, not just the picker) and
  remove the two-namespace mismatch permanently.

**Cons / risks:**
- Large lift for the value delivered: a new audio backend, a new
  dependency, and a rewrite of an already-working, already-shipped capture
  path (FEAT-VTT001) — capture itself has no known bug today, only
  enumeration for a picker UI does. Contradicts engineering-first-principles
  (KISS/YAGNI) — rejected as disproportionate to the problem.
- Not evaluated further; listed for completeness.

## Consequences

**If (a) is accepted:** the picker ships correctly for the common
PipeWire-desktop case, at the cost of a real, disclosed side effect
(VTT's mic choice becomes the system's mic choice). This needs explicit
operator sign-off precisely because "does selecting a mic in VTT change
what Zoom uses too" is a product-behaviour question, not an implementation
detail — hence Proposed, not Accepted, in this ADR.

**If (b) is accepted:** zero side effects, but the feature ships in a state
that cannot select the device most likely to motivate someone opening the
picker in the first place. Acceptable only if the operator explicitly
prefers "no side effects" over "device selection actually works on
PipeWire".

**If (c) is considered later:** would need its own ADR and a much larger
task; not proposed for near-term work.

## References

- TASK-VTT129 — Tray "Microphone" submenu — device picker UX (measured
  finding this ADR formalises)
- TASK-VTT062 — device-selection plumbing (`selected_device_index` →
  `resolve_device_ordinal`), already shipped
- STORY-VTT016 · DIRECT-VTT005 (cross-platform feature parity)
- `src/audio.rs::list_input_device_names`, `resolve_device_ordinal`
- `src/tray/linux.rs::get_default_mic_description` (existing `pactl`-based
  label, the enumeration shape alternative (a) would reuse)
- ADR-0005 — the tray-unification ADR this one borrows its
  proposed-pending-sign-off shape from
