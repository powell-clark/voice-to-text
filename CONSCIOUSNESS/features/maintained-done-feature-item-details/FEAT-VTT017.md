---
id: FEAT-VTT017
status: maintained
kano: performance
---

# FEAT-VTT017: large-v3-turbo and distil-large-v3 model support

## Description
The tray model menu offers `large-v3-turbo` and `distil-large-v3` as choices alongside `small` and `medium`. These newer models deliver better accuracy than the old `large-v3` at lower VRAM cost. The obsolete `medium.en`, `large-v3`, and legacy models are removed from the menu to reduce confusion.

## Acceptance Criteria
- [x] `large-v3-turbo` appears in the tray model submenu — verified in v2.0.x tray
- [x] `distil-large-v3` appears in the tray model submenu — verified
- [x] Obsolete model entries (medium.en, large-v1, large-v2) no longer appear — verified
- [x] Selecting `large-v3-turbo` triggers download if not cached, then loads and transcribes — verified in testing
- [x] SHA-256 hashes for large-v3-turbo and distil-large-v3 are correct upstream hashes, not placeholders — verified in `src/models.rs`

## Linked Tasks
- TASK-VTT010

## Parent Story
- STORY-VTT004
