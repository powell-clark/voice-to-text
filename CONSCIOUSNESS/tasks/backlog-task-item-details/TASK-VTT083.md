# TASK-VTT083: Story and task indexes still use 'epic_id' column header instead of 'directive_id' (naming-precision residue; drives fk-asymmetry validator warnings)

> **Needs review:** the agent created this task during real-time validation and is uncertain about scope or priority. Operator should review and re-tier as appropriate.


## Context

Auto-created from /consciousness:issue.

Report context:
Entity is DIRECTIVE (DIRECT-XX###) but the linkage column in STORY-*-INDEX.md and TASK-*-INDEX.md is still named 'epic_id'. Parser/CLI code already uses 'directive_id'/'directiveId' (28+5 refs in dist); the validator emits 'fk-asymmetry: ... directiveIds doesn't list ...' because data files say epic_id. Likely upstream PGPS schema rename (column header) requiring a schema-version bump + migration + reader alias for backward compat. Surfaced during the VTT directive split (DIRECT-VTT002/003/004). Earlier TASK-VTT065 renamed EPIC->DIRECT prefixes but left the column name.
transcripts:
  - chats/claude-code/2026-06-25/session-7871c871.jsonl

## Acceptance criteria

- [ ] Investigation confirms and documents whether the epic_id -> directive_id column rename belongs in the upstream consciousness-plugin schema or is local to this repo's PGPS data, with the decision and rationale recorded on this task
- [ ] If upstream: the decision records which upstream release/task carries the schema-version bump + migration, and no local rename is performed ahead of that release
- [ ] If local-only: STORY-*-INDEX.md and TASK-*-INDEX.md column headers are renamed from epic_id to directive_id (or directiveId), with a backward-compatible reader alias so pre-existing epic_id data still parses without error
- [ ] fk-asymmetry validator warnings referencing the epic_id/directiveId mismatch no longer appear against this repo's data (after the local fix ships, or after the upstream plugin release carrying the fix is adopted here)
- [ ] Any local schema/index-column change bumps CONSCIOUSNESS/stream/schema-version atomically per data--schema_version_discipline
