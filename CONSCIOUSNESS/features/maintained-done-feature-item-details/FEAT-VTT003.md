---
id: FEAT-VTT003
status: removed
superseded_by: FEAT-VTT029
kano: must-have
---

# FEAT-VTT003: macOS menu bar app with Cocoa native UI (removed — superseded)

## Description
**SUPERSEDED.** The original macOS UI was an Objective-C Cocoa menu bar app (`src/macos/*.m`). This code was deleted in TASK-VTT032 (7638 lines of C/ObjC removed in v2.0.0). The Rust macOS port (skeleton) exists in the codebase but has not been compiled, signed, or distributed.

**Successor:** FEAT-VTT029 (macOS .app bundle with accessibility permissions, in backlog).

## Why Done (not Maintained)
The ObjC source was deleted in TASK-VTT032. The Rust macOS skeleton exists but there is no distributed macOS binary. A user cannot install VTT on macOS today via any package manager. macOS distribution is tracked as STORY-VTT012 + FEAT-VTT029.

## Historical Acceptance Criteria
- [x] Cocoa menu bar icon appeared on macOS Monterey and Ventura — delivered in original Python version
- [x] ObjC source deleted from codebase in v2.0.0 — verified via TASK-VTT032
- [ ] Rust macOS binary distributed — NOT YET DONE (tracked in FEAT-VTT029)

## Linked Tasks
- TASK-VTT003, TASK-VTT020, TASK-VTT032

## Parent Story
- STORY-VTT001
