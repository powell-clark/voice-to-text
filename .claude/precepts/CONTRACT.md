# PGPS Contract

**Version:** 2.1 | **Updated:** 2025-11-27

This file is the single source of truth. pgps validates against this contract.
Every repo with consciousness has this file. Edit here, propagate everywhere.

---

## DEFINITIONS

D1. PGPS is the Project Global Positioning System showing project status.
D2. An Epic is a large initiative spanning multiple stories.
D3. A Story is a user-facing outcome with measurable SMART criteria.
D4. A Task is a single unit of work within a story.
D5. A Todo is a session-level item managed by Claude Code, not stored in CONSCIOUSNESS.
D6. A Feature is a shipped capability in the product.
D7. An Active section contains items in flight with a status column.
D8. A Done section contains completed archived items.
D9. A Backlog section contains future items with NO status column.
D10. A Project code is a 2-character identifier (CC, JJ, NB).
D11. SMART means Specific, Measurable, Achievable, Relevant, Time-bound.

---

## RULES

### Statuses

R1. Work status must be: planned|in-progress|in-review|blocked|done|cancelled|duplicate
R2. Feature status must be: planned|in-progress|in-review|done|deprecated

### Categories & Priorities

R3. Epic category must be: must-have|should-have|nice-to-have|performance|delighter|future
R4. Epic priority must be: p0|p1|p2|p3|p4

### File Headers

R5. EPICS.md: id|quarter|stories|category|priority|title|completion|created|started|expected_end|actual_end [11 cols]
R6. EPIC-BACKLOG.md: id|quarter|stories|category|priority|title|completion|created|started|expected_end|actual_end [11 cols]
R7. EPIC-DONE.md: id|quarter|stories|category|priority|title|completion|created|started|expected_end|actual_end [11 cols]
R8. STORY.md: id|epic|tasks|status|title [5 cols]
R9. STORY-BACKLOG.md: id|epic|tasks|title [4 cols, NO status]
R10. STORY-DONE.md: id|epic|tasks|status|title [5 cols]
R11. TASKS.md: id|story|status|title [4 cols]
R12. TASK-BACKLOG.md: id|story|title [3 cols, NO status]
R13. TASK-DONE.md: id|story|status|title [4 cols]
R14. FEATURES.md: id|status|location|tested|description [5 cols]
R15. FEATURES-BACKLOG.md: id|location|tested|description [4 cols, NO status]
R16. FEATURES-DONE.md: id|status|location|tested|description [5 cols]

### Required Files

R17. Required files: EPICS.md, STORY.md, TASKS.md, FEATURES.md, TODO.md
R18. Optional files: all -BACKLOG.md and -DONE.md variants

### SMART Patterns

R19. Epics and Stories must match at least one: \d+%|\d+ms|\d+s|<\d+|>\d+|\d+-\d+|\d+ to \d+|within \d+|under \d+|every \d+|per \d+|\d+\+|\d+ (files?|tests?|hooks?|sessions?|stories?|tasks?)

### Trajectory

R20. In-progress story must have tasks column populated
R21. In-progress story must have at least one in-progress task

### Commits

R22. Commits cannot contain: Generated with [Claude Code]
R23. Commits cannot contain: Co-Authored-By: Claude
R24. Commits cannot contain: noreply@anthropic.com

### ID Format

R25. IDs must match: ^(EPIC|STORY|TASK|FEAT)-[A-Z]{2}\d{3}$

### Display Order

R26. Sections: EPICS,EPICS.DONE,EPICS.BACKLOG,STORY,STORY.DONE,STORY.BACKLOG,TASKS,TASKS.DONE,TASKS.BACKLOG,TODO,FEATURES,FEATURES.DONE,FEATURES.BACKLOG,GIT,CURRENT
R27. Status sort: in-review,in-progress,planned,blocked

### Section Behaviour

R28. Active sections display status column
R29. Done sections display completed items
R30. Backlog sections have NO status column
R31. TODO.md managed externally by Claude Code

---

## VALIDATION OUTPUT

PASSED: All rules pass
FAILED: Rule R## failed - expected: X, got: Y, file: Z, line: N

---

11 definitions. 31 rules. One contract.
