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
- [x] Auto-repeat events are identified by comparing event timestamps and repeat flag — verified in source (`src/hotkey.rs`)
- [x] Filtering does not introduce perceptible delay between physical keypress and recording start — verified subjectively in daily use

## Linked Tasks
- TASK-VTT013

## Parent Story
- STORY-VTT006
