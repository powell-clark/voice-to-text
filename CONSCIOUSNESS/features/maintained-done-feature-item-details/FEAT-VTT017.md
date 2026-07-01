---
id: FEAT-VTT017
status: maintained
kano: performance
---

# FEAT-VTT017: large-v3-turbo and distil-large-v3 model support

## Description
The tray model menu offers `large-v3-turbo` alongside `small`, `medium`, and `large-v3`. The newer turbo model delivers better accuracy at lower VRAM cost. Obsolete `large-v1`/`large-v2` entries are gone; the legacy `distil-large-v3` name auto-migrates to `large-v3-turbo` rather than appearing as its own menu entry.

## Acceptance Criteria
- [x] **AC-1** — `large-v3-turbo` appears in the tray model submenu — verified in v2.0.x tray
- [x] **AC-2** — `distil-large-v3` / `distil-large-v3.5` legacy names auto-migrate to `large-v3-turbo` (`src/main.rs`) rather than appearing as a separate menu entry — verified
- [x] **AC-3** — Obsolete `large-v1` / `large-v2` entries no longer appear; the menu is small.en, small, medium.en, medium, large-v3-turbo, large-v3 — verified in `src/models.rs` MODELS
- [x] **AC-4** — Selecting `large-v3-turbo` triggers download if not cached, then loads and transcribes — verified in testing
- [ ] **AC-5** — [deferred → TASK-VTT112] Stored per-model SHA-256 allowlist verification — NOT implemented; `src/models.rs` computes and logs the download hash but does not check it against an expected constant

## Linked Tasks
- TASK-VTT010

## Parent Story
- STORY-VTT004
