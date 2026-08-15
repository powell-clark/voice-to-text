<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Director handoff

Director sessions share state across repos through an append-only JSONL log at CONSCIOUSNESS/stream/director-context.jsonl; the Stop hook writes on leave, the SessionStart hook reads on enter; only Director sessions write or read the handoff log.

## Narrative

The log lives at CONSCIOUSNESS/stream/director-context.jsonl, under stream/
with the other append-only event logs. This rule named the top-level path
CONSCIOUSNESS/director-context.jsonl until 2026-08-10, which was the layout
migration 20260511000000000_relocate_loose_files retired — that migration
moves director-context.jsonl off the root precisely so it stops living there.

The stale path was load-bearing rather than cosmetic. A neurologist pass in a
consumer repo read this rule, found a top-level director-context.jsonl that
the allowlist rejects, and concluded the allowlist was wrong and the precept
non-negotiable (GitHub #1453). Both halves were backwards: the implementation
had always written to stream/, the allowlist covers stream/ already, and the
top-level file was the residue the migration exists to clear.

## Requires

- MUST write a leave entry from the Stop hook on every turn end when the actor's role is director
- MUST read pending handoffs from the SessionStart hook for director sessions and auto-append a matching enter entry threaded by context_id (UUID generated at leave time)
- MUST scan the local repo plus every sibling listed in config.json federation.siblings when computing pending-handoff state
- MUST pin schema_version to 1 in every handoff entry
- MUST treat all handoff writes as best-effort — failures log to stderr, never throw

## Forbids

- MUST NOT write or read the handoff log from non-Director sessions
- MUST NOT mutate previously-written entries — the log is strictly append-only
- MUST NOT block the turn lifecycle on handoff write failure
- MUST NOT skip the context_id thread — every leave must carry a UUID and every matching enter must reuse it

## References

- precept:precept_specification
- doc:CONSCIOUSNESS/directives/active-directive-item-details/DIRECT-CCC021
- doc:CONSCIOUSNESS/stream/director-context.jsonl

## Verified by

packages/core/session/director-context.ts:appendDirectorLeave
