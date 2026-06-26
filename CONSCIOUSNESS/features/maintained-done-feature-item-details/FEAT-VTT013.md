---
id: FEAT-VTT013
status: maintained
kano: must-have
---

# FEAT-VTT013: X11 key auto-repeat filtering

## Description
When the user holds the push-to-talk hotkey down, X11 generates repeated keypress events. Without filtering, VTT would start and stop recording on every synthetic repeat event, producing garbled output and unexpected behaviour. The filter detects and discards auto-repeat keypresses, so only the first physical press starts recording and only the physical release stops it.

## Acceptance Criteria
- [x] Holding the push-to-talk key (default: F4) for 5 seconds produces exactly one recording — not multiple — verified in TASK-VTT013 and v2.0.0 daily use
- [x] Release of the key stops the recording at the correct moment regardless of how long the key was held — verified
- [x] Auto-repeat events are identified and discarded — verified in source (`src/hotkey/linux.rs`)
- [x] Filtering does not introduce perceptible delay between physical keypress and recording start — verified subjectively in daily use

## Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)
Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 6 — hotkey auto-repeat handling).

**🐧 Linux — ✅ works** (this card)
- [x] Held hotkey = exactly one recording — X11 `XkbSetDetectableAutoRepeat` + manual KeyPress/KeyRelease pairing — `src/hotkey/linux.rs`

**🍎 macOS — ✅ works**
- [x] `rdev` listener filters auto-repeat; held key = one recording — `src/hotkey/portable.rs` (TASK-VTT100)
- [ ] Requires Input-Monitoring permission (no prompt flow without `.app`, FEAT-VTT029)

**🪟 Windows — ✅ works**
- [x] `rdev` listener filters auto-repeat; live keycode remap via SetKeycode — `src/hotkey/portable.rs` (TASK-VTT100)

## Linked Tasks
- TASK-VTT013

## Parent Story
- STORY-VTT006
