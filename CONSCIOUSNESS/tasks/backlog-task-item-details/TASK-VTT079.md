---
id: TASK-VTT079
status: backlog
priority: p2
title: Upstream — feature-index status column splits PGPS display but fails validator (dist build inconsistency)
story_ids: []
epic_id: DIRECT-VTT002
feature_ids: []
---

# TASK-VTT079: Upstream consciousness dist-build inconsistency — feature terminal index status column

## Problem
ADR roadmap--entity_lifecycle_graph (schema 20260621090000000) says features terminate in a
single status-distinguished index `FEATURE-MAINTAINED-DONE-INDEX.md` with columns
`id|kano|status|description|story_ids|task_ids|doc`. The compiled dist build is internally split:

- The **PGPS display script** (`main.js`) reads a `status` column to split FEATURES.MAINTAINED vs
  FEATURES.DONE. With a 6-column index (no status) it double-lists every feature in BOTH buckets.
- The **validator** (Rule 31 schema + Rule 33 FK) rejects the 7-column status format: it parses the
  index with the legacy 6-column map `id|kano|description|story_ids|task_ids|doc`, so the status
  column shifts every field right by one and produces ~60 false "Invalid reference ID" errors.

Measured in this repo (dist `main.js`, 2026-06-23):
- 6-col index (no status): 51/52 validation passed, 0 feature errors, BUT /pgps double-lists.
- 7-col index (with status, matches schema.js line 145 columns_done): 49/52, 65 errors.

So from a consumer repo you cannot have BOTH clean validation AND the correct maintained/done
display split. Plugin install must not be modified from the consumer (safety precept), so the fix
is upstream.

## Decision taken in VTT (2026-06-23)
Kept the validator-conformant 6-col index. The maintained/removed/deprecated distinction is preserved
in (a) each feature card's `status:` frontmatter and (b) a `[status]` prefix in the index description.
The /pgps two-bucket display split is deferred until this upstream fix lands.

## Acceptance Criteria
- [ ] Upstream consciousness validator accepts the 7-column `id|kano|status|description|story_ids|task_ids|doc` feature terminal index (Rule 31/33 use columns_done, not the legacy 6-col map)
- [ ] With the status column present, PGPS validation passes AND the display splits MAINTAINED vs DONE
- [ ] Once shipped, VTT re-adds the status column to FEATURE-MAINTAINED-DONE-INDEX.md and drops the `[status]` description prefixes

## Notes
Report to the consciousness plugin (use /consciousness:issue). Related to TASK-VTT065 (the partial
feature-index migration) and TASK-VTT078 (folder collapse + card statuses, done).

## Update 2026-06-27 — confirmed still present in consciousness v0.42.9
Re-tested from VTT. The split-brain persists, now more precisely diagnosed:
- `schema.json` HAS been updated — `index_schemas.features.columns_done` =
  `[id, kano, status, description, story_ids, task_ids, doc, last_tested]` (status column defined).
- The **display** script (`main.js`) honours it — an 8-col index with `status` splits
  FEATURES.MAINTAINED (status=maintained) from FEATURES.DONE (status!=maintained) correctly.
- The **validator** (Rule 31/33) still parses the feature terminal index with the legacy 6-col
  map `id|kano|description|story_ids|task_ids|doc`, so the 8-col index shifts every field right and
  produces ~48 false "Invalid reference ID" errors (validation 49/52, was 51/52).
- Net: from a consumer you still cannot have BOTH clean validation AND the maintained/done split.
- **Refined upstream fix:** point validator Rule 31/33 at `schema.columns_done` for the feature
  terminal index (the column data already exists in schema.json) — backward-compatible if it accepts
  either the 6-col or 8-col header. VTT then adopts the status column and drops the `[tag]` prefixes.
