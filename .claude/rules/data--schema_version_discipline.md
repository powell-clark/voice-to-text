<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Data schema version discipline

Every change to CONSCIOUSNESS/schema.json bumps the marker file
CONSCIOUSNESS/.schema-version atomically. Both files carry the same
value. Drift between them is a structural integrity failure that
must be repaired before further schema work proceeds.

The marker is the canonical source of truth for "what schema does
this repo currently honour". The schema.json field documents intent
and is what tooling reads. They MUST agree.

Discovered 2026-05-06: the marker had been stale 5 months while
schema.json actively churned (TASK-CCC29015 feature lifecycle,
story_points column, EPIC->DIRECTIVE rename, advisory-board fixes).
PGPS reported a 5-month-stable schema version while the underlying
file format was changing. That is exactly the silent rot S-1 was
meant to prevent. Manual realignment landed; this precept locks
the gate so it cannot recur.

## Scope

universal

## Marker format

### Pattern

^[0-9]{17}(_[A-Za-z0-9_-]+)?$

### Description

17-digit timestamp YYYYMMDDHHMMSSmmm, optionally followed by an
underscore-separated description. The 17-digit prefix sorts
lexically as it sorts chronologically — useful for diff readers
and audit reconstruction.

### Examples

- 20251209233807764_add_commentary_steering_files
- 20260506190440587_align_with_schema_state_post_drift
- 20260507120000000

## Bump required when

- Adding, removing, or renaming a column in any INDEX header
- Changing the allowed values of an enum field (status, priority, kano, etc.)
- Adding, removing, or renaming a top-level entity type (DIRECTIVE, STORY, TASK, FEATURE, REVIEW)
- Changing the YAML/Markdown frontmatter requirements on a detail card
- Changing the JSONL schema for steering, commentary, task-events, handoffs, etc.
- Any structural change to schema.json content beyond formatting/comments

## Bump not required when

- Comment-only edits to schema.json
- Whitespace-only edits
- Reordering keys without changing values (when consumers don't depend on key order)

## Migration doc

### Required when breaking

true

### Location

CONSCIOUSNESS/MIGRATIONS.md (when authored)

### Format

For every breaking schema change, append:
  ## {marker-value}
  - Date: YYYY-MM-DD
  - Change: brief description
  - Migration: how consumers update (or "no consumer action required")
  - Affected: list of INDEX files / detail cards / JSONL stores impacted

## References

### Directive

DIRECT-CCC029

### Discipline gate

S-1

### Discovery task

TASK-CCC29061

### Discovery context

2026-05-06 evening, operator skepticism led to investigation

### Related

- TASK-CCC29033 (append-task-cli ID counter desync — sibling silent-drift bug)
