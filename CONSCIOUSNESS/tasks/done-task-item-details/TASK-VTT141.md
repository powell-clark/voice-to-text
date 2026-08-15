# TASK-VTT141: Repair corrupted directive/story INDEX column alignment

## Context

Neurologist diagnostic (2026-07-18) found an uncommitted, incomplete hand-edit across 4 INDEX files: header row column changes not matched by data-row rewrites, producing 286 PGPS validation errors. Files: STORY-FULFILLED-REJECTED-INDEX.md (header gained status+priority), DIRECT-ACTIVE-INDEX.md and DIRECT-BACKLOG-INDEX.md (header dropped quarter), DIRECT-MAINTAINED-DONE-INDEX.md (header dropped quarter, gained status). Directive files need only a mechanical header fix (quarter column re-added, data already correct). Story file needs 8 inferred status/priority values pending operator confirmation. Also covers the already-committed cleanup this task retroactively traces: task-card moves (TASK-VTT023/060/062/109/123/132) to match INDEX status, and removal of stale CONSCIOUSNESS/schema.json + CONSCIOUSNESS/.schema-version markers superseded by CONSCIOUSNESS/stream/schema-version (commit 71edaf7).

## Acceptance criteria

- [ ] _(to be filled in)_

## Closed without contract

This task reached a terminal state while its acceptance criteria still read
`_(to be filled in)_`. No criteria have been authored retrospectively: the contract
that would have governed this work was never written, and this marker records that
honestly rather than manufacturing one after the fact.
