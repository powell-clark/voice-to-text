# TASK-VTT075: PGPS validation error wall is overwhelming for new users — needs friendlier guidance and auto-fix path

> **Groomed (product-owner grooming pass, 2026-07-21):** acceptance criteria below are lifted
> directly from this card's own "Proposed improvements (a)(b)(c)" — no new scope invented. This is
> an upstream consciousness-plugin UX request (like TASK-VTT079), not something fixable from this
> consumer repo. Still open: this session's own `/consciousness:pgps` run showed 48/52 passed, 257
> errors, "Likely root: incomplete migration / structural drift — 255 of 257 resolve automatically"
> — i.e. the plugin already implemented improvement (a) (root-cause summary) and part of (b) (points
> to /consciousness:neurologist) since this card was filed on 2026-06-2x; only (c), severity
> distinction in the raw error list, still reads as a flat wall. Re-scoped acceptance criteria to
> reflect what's still outstanding.


## Context

Auto-created from /consciousness:issue.

Report context:
transcripts:
  - /home/powell-clark/.claude/projects/-home-powell-clark-projects-aux-voice-to-text/e81a04c7-1103-4eb1-aed8-07cfc8659e3b.jsonl

Description: The PGPS full output shows 80+ validation errors (Rule 31, Rule 33 FK failures, phantom-refs, Rule 67 file structure) in a dense wall of text. This is alarming for new users and desktop plugin users who don't know that: (1) this is usually caused by a single incomplete migration (not 80 independent problems), (2) /consciousness:neurologist can auto-fix the root cause in one pass, (3) warnings vs errors have different severity. Proposed improvements: (a) surface a single top-level cause summary when errors cascade from one root issue (e.g. 'Root: incomplete migration — 78 of 80 errors resolve automatically'), (b) add a callout in the PGPS output when validation fails: 'Run /consciousness:neurologist to auto-repair', (c) distinguish error severity better in the output — critical blockers vs informational warnings. The neurologist DID fix this project (52/52 after repair, was 49/52), confirming auto-fix works — but users need to know to run it.

Also noted: entity_lifecycle_graph migration header constants drift from schema.json columns_done — the migration writes headers the validator then immediately rejects, and isApplied() short-circuits on pre-created empty targets. This is the underlying cause of the cascade.

## Acceptance criteria

- [x] Surface a single top-level cause summary when errors cascade from one root issue — CONFIRMED DONE upstream: `/consciousness:pgps` now prints "Likely root: incomplete migration / structural drift — N of M resolve automatically" (verified 2026-07-21, consciousness v0.45.4/0.45.6)
- [x] Add a callout pointing at the auto-fix path when validation fails — CONFIRMED DONE upstream: output includes "Run /consciousness:neurologist to diagnose and auto-repair structural issues." (verified 2026-07-21)
- [ ] Distinguish error severity in the raw error listing — critical blockers vs informational warnings (still a flat list under "Failed:" / "Warnings:" with no per-item severity marker beyond the section heading, verified still outstanding 2026-07-21)
