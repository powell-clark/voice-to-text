# TASK-VTT075: PGPS validation error wall is overwhelming for new users — needs friendlier guidance and auto-fix path

> **Needs review:** the agent created this task during real-time validation and is uncertain about scope or priority. Operator should review and re-tier as appropriate.


## Context

Auto-created from /consciousness:issue.

Report context:
transcripts:
  - /home/powell-clark/.claude/projects/-home-powell-clark-projects-aux-voice-to-text/e81a04c7-1103-4eb1-aed8-07cfc8659e3b.jsonl

Description: The PGPS full output shows 80+ validation errors (Rule 31, Rule 33 FK failures, phantom-refs, Rule 67 file structure) in a dense wall of text. This is alarming for new users and desktop plugin users who don't know that: (1) this is usually caused by a single incomplete migration (not 80 independent problems), (2) /consciousness:neurologist can auto-fix the root cause in one pass, (3) warnings vs errors have different severity. Proposed improvements: (a) surface a single top-level cause summary when errors cascade from one root issue (e.g. 'Root: incomplete migration — 78 of 80 errors resolve automatically'), (b) add a callout in the PGPS output when validation fails: 'Run /consciousness:neurologist to auto-repair', (c) distinguish error severity better in the output — critical blockers vs informational warnings. The neurologist DID fix this project (52/52 after repair, was 49/52), confirming auto-fix works — but users need to know to run it.

Also noted: entity_lifecycle_graph migration header constants drift from schema.json columns_done — the migration writes headers the validator then immediately rejects, and isApplied() short-circuits on pre-created empty targets. This is the underlying cause of the cascade.

## Acceptance criteria

- [ ] _(to be filled in)_
