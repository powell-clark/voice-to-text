# TASK-VTT142: Repair FEATURE-MAINTAINED-DONE-INDEX column alignment

## Context

Same corruption shape as TASK-VTT141 but a different file, found via self-healing --dry-run --verbose after the TASK-VTT141 fix landed. CONSCIOUSNESS/features/FEATURE-MAINTAINED-DONE-INDEX.md header is 'id|kano|status|description|story_ids|task_ids|doc|last_tested' (8 columns) but data rows only carry 6 fields ('id|kano|description|story_ids|task_ids|doc') — status and last_tested are missing from every row, so the parser misreads story_ids as description/status overflow and shifts everything right. Self-healing flags 85 phantom-ref items across ~24 features that are actually this same misalignment, not real broken FKs. Needs the same treatment as TASK-VTT141: mechanical header fix is NOT enough here since status is missing data, not a relocated column — will need per-row status values inferred/confirmed (most rows already say [maintained] or [removed ...] in the description text, which may be usable signal) before self-healing --apply is safe to run.

## Acceptance criteria

- [ ] _(to be filled in)_
