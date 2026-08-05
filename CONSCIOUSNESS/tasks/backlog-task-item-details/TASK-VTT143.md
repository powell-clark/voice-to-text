# TASK-VTT143: Rule 63 maintained-directive false positive

## Context

PGPS validation rule 63 (packages/core/pgps/validators/semantic-validator/index.js loadIdsByStatus) buckets directive completion status by which INDEX file/folder a row lives in, not by its actual status column. DIRECT-VTT002 moved active -> maintained (2026-07-21, operator request, daily-driver on Linux) into the combined DIRECT-MAINTAINED-DONE-INDEX.md; rule 63 now reads it as folder-status 'done' and flags its active/backlog child stories (STORY-VTT018 active; STORY-VTT019/014/015/016/017/009/008 backlog) as an error, even though status=maintained legitimately keeps ongoing per-release work open (same pattern already accepted for maintained features, e.g. FEAT-VTT038/FEAT-VTT026). Non-blocking (PGPS display and FK integrity are unaffected) but produces a permanent false Rule 63 error for any directive that reaches maintained while still doing upkeep work. Fix belongs upstream in the consciousness plugin (loadIdsByStatus / rule 63 needs to read the status column, not the folder), not in this repo's CONSCIOUSNESS/ data — report via /consciousness:issue or file against the plugin repo.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
