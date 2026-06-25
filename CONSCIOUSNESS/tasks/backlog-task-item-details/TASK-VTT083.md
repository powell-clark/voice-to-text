# TASK-VTT083: Story and task indexes still use 'epic_id' column header instead of 'directive_id' (naming-precision residue; drives fk-asymmetry validator warnings)

> **Needs review:** the agent created this task during real-time validation and is uncertain about scope or priority. Operator should review and re-tier as appropriate.


## Context

Auto-created from /consciousness:issue.

Report context:
Entity is DIRECTIVE (DIRECT-XX###) but the linkage column in STORY-*-INDEX.md and TASK-*-INDEX.md is still named 'epic_id'. Parser/CLI code already uses 'directive_id'/'directiveId' (28+5 refs in dist); the validator emits 'fk-asymmetry: ... directiveIds doesn't list ...' because data files say epic_id. Likely upstream PGPS schema rename (column header) requiring a schema-version bump + migration + reader alias for backward compat. Surfaced during the VTT directive split (DIRECT-VTT002/003/004). Earlier TASK-VTT065 renamed EPIC->DIRECT prefixes but left the column name.
transcripts:
  - chats/claude-code/2026-06-25/session-7871c871.jsonl

## Acceptance criteria

- [ ] _(to be filled in)_
