<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Review gates

Items cannot advance from in_review to done without a qualifying approval. The gate derives per entity from reversibility (ADR review_gates--reversibility_derived_model): reversible entities (task, story, directive) auto-close; features gate for a human. Verdict paths: (1) auto-close — the entity's per-entity gate is auto-flow, agent records a bypass-approved verdict and marks done; (2) machine-approved — N agent-approved verdicts auto-stamp after review_rounds_before_human cycles; (3) human — a human approves explicitly. The blanket review_gates.bypass flag is retired.

## Narrative

The auto-close path procedure (when the entity's per-entity gate is auto-flow —
task / story / directive by default):
  1. Move the item to in_review in the INDEX (update status column)
  2. Append a PSV row to CONSCIOUSNESS/reviews/REVIEW-INDEX.md:
       REVIEW-CCC{n}|task|TASK-CCC{n}|agent|{session_id}|bypass-approved|1|{ISO-timestamp}|auto-close: entity gate is auto-flow
  3. Move the item to done in the INDEX (update status column)
  4. Claim next task and continue — do not pause for human confirmation

Authority. Auto-close permission is derived per entity by isEntityAutoFlow
(packages/core/review/auto-close.ts) from the per-entity gate (DEFAULT_GATES /
review-gates entity_overrides), NOT from a global flag. An entity is auto-flow
when its resolved gate is not a hard gate (requires: auto-approve, or gate:
warn/off); features and hard-gated kano tiers require a human verdict. This is
the L3 layer of ADR review_gates--reversibility_derived_model. The L1 hard-block
layer (dangerous commands, append-only files, active-ADR mutations) is a separate
security surface and is unaffected.

Back-compat. The blanket review_gates.bypass flag is retired as the authority. A
legacy review_gates.bypass: false still present in a repo's config.json is
honoured as a deprecated global "gate everything" override (version-gated so
behaviour does not silently change on upgrade); a deprecation notice points the
operator at the per-entity model. Removing the flag adopts the reversibility model.

REVIEW-INDEX.md columns: id|target_type|target_id|reviewer_type|reviewer_id|verdict|iteration|reviewed_at|notes
Create the file with the header row if it does not exist.

## Requires

- MUST record every verdict in CONSCIOUSNESS/reviews/REVIEW-INDEX.md as a PSV row with the nine columns above
- MUST follow the three-step auto-close procedure when the entity's per-entity gate is auto-flow — move to in_review, write verdict, move to done
- MUST cap agent verdicts to pending-review, agent-approved, agent-rejected, bypass-approved
- MUST allow autonomous backlog to in_progress transitions without a review verdict (claim discipline only, no quality gate)
- MUST create CONSCIOUSNESS/reviews/REVIEW-INDEX.md with header row if it does not exist
- MUST gate features and hard-gated entities for a human verdict — auto-close applies only to reversible auto-flow entities

## Forbids

- MUST NOT wait for human approval when the entity's per-entity gate is auto-flow — waiting stalls the autonomous loop
- MUST NOT skip writing the verdict row — the audit trail is required even on auto-close
- MUST NOT jump pending or in_progress straight to done without passing through in_review
- MUST NOT override a human rejection with subsequent machine or agent verdicts
- MUST NOT reintroduce the blanket review_gates.bypass flag as the auto-close authority — auto-close derives per entity from reversibility

## References

- precept:task_lifecycle
- precept:precept_specification
- adr:review_gates--reversibility_derived_model
- doc:CONSCIOUSNESS/reviews/REVIEW-INDEX.md

## Verified by

packages/core/review/approve/cli.ts:main
