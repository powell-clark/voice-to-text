---
id: TASK-VTT078
status: done
priority: p1
title: Feature terminal index conformance — single folder, precise card statuses
story_ids: []
epic_id: DIRECT-VTT002
feature_ids: []
---

# TASK-VTT078: Feature terminal index conformance

## Description
Conform the feature terminal state to ADR roadmap--entity_lifecycle_graph:
- One folder `maintained-done-feature-item-details/` (collapse the two non-canonical folders)
- Index `FEATURE-MAINTAINED-DONE-INDEX.md` carries the `status` column (id|kano|status|description|story_ids|task_ids|doc)
- Stamp precise terminal status per feature: maintained (19), removed (FEAT-VTT002/003/009), deprecated (FEAT-VTT007)

Fixes the /pgps double-listing of all features in both MAINTAINED and DONE (caused by the missing status column).

## Acceptance Criteria
- [x] All 23 cards live in maintained-done-feature-item-details/
- [x] The two non-canonical folders (done-feature-item-details, maintained-feature-item-details) are removed
- [~] FEATURE-MAINTAINED-DONE-INDEX.md status column — DEFERRED to TASK-VTT079: the dist validator rejects the status column (enforces legacy 6-col); kept 6-col for clean validation, status carried in card frontmatter + `[status]` description prefix instead
- [x] Each card frontmatter status matches the intended terminal state (maintained / removed / deprecated)
- [~] /pgps display split — DEFERRED to TASK-VTT079 (display reads status column the validator forbids); validation is clean at 52/52

## Outcome
- Single canonical folder achieved; structure now schema-correct
- Precise per-feature status in card frontmatter: 19 maintained, FEAT-VTT002/003/009 removed, FEAT-VTT007 deprecated
- Validation 52/52 clean
- The status-column display split is blocked on the upstream dist-build inconsistency (TASK-VTT079)
