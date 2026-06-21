---
id: TASK-VTT066
status: in_progress
priority: p2
title: Repo structure tidy — remove empty stale CONSCIOUSNESS dirs and build artifacts
story_ids: []
epic_id: DIRECT-VTT002
feature_ids: []
blocked_by: []
blocks: []
assignee: ""
parent_task_id: ""
sequence: 3
---

## What

Remove empty stale directories from CONSCIOUSNESS/ left behind by past migrations,
and clean gitignored build artifacts from the pre-Rust ObjC era.

## Why

The repository accumulated empty directories from two migrations:
1. `roadmap/` (epics-*, stories-*, tasks-*, features-* subdirs) — superseded when the
   Consciousness plugin renamed "epics" to directives and split the roadmap into
   per-entity directories (`directives/`, `stories/`, `tasks/`, `features/`).
2. `architectural-decision-records/` and `architectural-decisions/` — superseded by `adr/`.
3. `observations/` — never populated.

The `build/` directory contains 9 `.o` files from the pre-v2.0 Objective-C macOS build.
Cargo replaced all of this in v2.0.0 (TASK-VTT032). The dir is gitignored.

## Acceptance criteria

- [x] Empty stale dirs removed from CONSCIOUSNESS/ (roadmap/, architectural-decision-records/,
      architectural-decisions/, observations/)
- [x] build/ artifacts cleaned
- [x] No git-tracked files deleted
- [x] Active task index updated; review verdict written

## Notes

- `CONSCIOUSNESS/concepts/` — kept: has README.md (git-tracked), valid concepts directory
- `build-archives/` — already .gitignored (line 74), not git-tracked; left as local cache
- `vtt-linux.prebuilt` — intentionally committed (v2.1.0 release binary)
