<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Director handoff

Director sessions share state across repos through an append-only JSONL log at CONSCIOUSNESS/director-context.jsonl; the Stop hook writes on leave, the SessionStart hook reads on enter; only Director sessions write or read the handoff log.

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
- doc:CONSCIOUSNESS/director-context.jsonl

## Verified by

packages/core/session/director-context.ts:appendDirectorLeave
