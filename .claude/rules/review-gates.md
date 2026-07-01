<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Review gates

Items cannot advance from in_review to their terminal state without a qualifying approval. The gate derives per entity from reversibility (ADR review_gates--reversibility_derived_model) across three approval tiers — auto-approve (tasks, stories, directives by default), agent (configurable min_agent_reviews), human (features with must-have/performance kano by default). A story fulfils when its owning features reach maintained, gated by the feature review gates, not by a separate story in_review step. The blanket review_gates.bypass flag is retired.

## Narrative

Three approval tiers, not a binary machine-vs-human model:
  auto-approve — the entity's gate is off or requires: auto-approve; the agent
                 records a bypass-approved verdict and closes the item
  agent        — N independent agent verdicts required (min_agent_reviews); the
                 agent records agent-approved
  human        — the operator must explicitly approve; the agent records
                 human-pending and waits

The gate resolves per entity type, and per kano tier for features:
  must-have feature             → human / hard gate
  performance feature           → agent / hard gate
  delighter / nice-to-have feature → agent / warn gate (non-blocking)
  task / story / directive      → auto-approve / off gate (DEFAULT_GATES; the
                                  operator can override via entity_overrides)

Story lifecycle note. Stories do not carry their own in_review quality gate. A
story fulfils when its owning features move to maintained — the feature review
gates carry the check. The story transitions accepted / in_progress → fulfilled
and lands in STORY-FULFILLED-REJECTED-INDEX.md with status=fulfilled; a rejected
story lands in the same index with status=rejected. Both are terminal-but-reopenable
per ADR roadmap--entity_lifecycle_graph: a new or un-met acceptance criterion
reopens the story while its owning feature stays maintained.

Auto-close path (when the resolved gate is auto-approve):
  1. Move the item to in_review in the INDEX (update status column)
  2. Append a PSV row to CONSCIOUSNESS/reviews/REVIEW-INDEX.md:
       REVIEW-CCC{n}|task|TASK-CCC{n}|agent|{session_id}|bypass-approved|1|{ISO-timestamp}|auto-close: entity gate is auto-approve
  3. Move the item to its terminal state in the INDEX (done for tasks; for
     directives/features the MAINTAINED-DONE index with status done|maintained)
  4. Continue — do not pause for human confirmation

Authority. isEntityAutoFlow (packages/core/review/auto-close.ts) derives permission
from the resolved per-entity gate (DEFAULT_GATES / review-gates entity_overrides),
NOT from the retired blanket bypass flag. A legacy review_gates.bypass: false still
present in a repo's config.json is honoured only as a deprecated global "gate
everything" override (version-gated so behaviour does not silently change on
upgrade); removing it adopts the per-entity model. The L1 hard-block layer
(dangerous commands, append-only files, active-ADR mutations) is a separate
security surface and is unaffected.

REVIEW-INDEX.md columns: id|target_type|target_id|reviewer_type|reviewer_id|verdict|iteration|reviewed_at|notes
Create the file with the header row if it does not exist.

## Requires

- MUST record every verdict in CONSCIOUSNESS/reviews/REVIEW-INDEX.md as a PSV row with the nine columns above
- MUST follow the three-step auto-close procedure when the entity's gate is auto-approve — move to in_review, write the bypass-approved verdict, move to the terminal state
- MUST cap agent verdicts to pending-review, agent-approved, agent-rejected, bypass-approved
- MUST allow autonomous backlog to in_progress transitions without a review verdict (claim discipline only, no quality gate)
- MUST create CONSCIOUSNESS/reviews/REVIEW-INDEX.md with header row if it does not exist
- MUST gate features and hard-gated entities for the resolved tier (agent or human) — auto-close applies only when the resolved gate is auto-approve
- MUST treat story fulfillment as gated by the owning feature review gates, not by a separate story in_review step

## Forbids

- MUST NOT wait for human approval when the entity's gate is auto-approve — waiting stalls the autonomous loop
- MUST NOT skip writing the verdict row — the audit trail is required even on auto-close
- MUST NOT jump pending or in_progress straight to a terminal state without passing through in_review
- MUST NOT override a human rejection with subsequent agent verdicts
- MUST NOT reintroduce the blanket review_gates.bypass flag as the auto-close authority — auto-close derives per entity from reversibility
- MUST NOT describe the gate model as binary machine-vs-human — it is three-tier: auto-approve, agent, human

## References

- precept:task_lifecycle
- precept:precept_specification
- adr:review_gates--reversibility_derived_model
- adr:roadmap--entity_lifecycle_graph
- doc:CONSCIOUSNESS/reviews/REVIEW-INDEX.md

## Verified by

packages/core/review/approve/cli.ts:main
