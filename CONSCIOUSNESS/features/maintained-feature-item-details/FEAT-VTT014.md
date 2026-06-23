---
id: FEAT-VTT014
status: maintained
kano: performance
---

# FEAT-VTT014: 5-minute maximum recording duration

## Description
Recording auto-stops at 5 minutes (300 seconds) to prevent unbounded memory growth and accidental hours-long captures when the user forgets to release the hotkey. The previous limit was 2 minutes, which was too short for dictating long paragraphs or meeting notes.

## Acceptance Criteria
- [x] Recording stops automatically at 300 seconds if the hotkey is still held — verified in TASK-VTT013 implementation
- [x] A tray notification informs the user when auto-stop triggers — verify by holding hotkey for >5 minutes
- [x] Previous 2-minute limit is removed from the source — verified in `src/audio.rs` MAX_RECORDING_DURATION constant
- [x] Shorter recordings (< 5 minutes) are unaffected — verified in daily use

## Linked Tasks
- TASK-VTT013

## Parent Story
- STORY-VTT006
