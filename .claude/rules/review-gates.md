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
  2. Record the verdict with the fragment write CLI (append-verdict-cli), or
     through a sanctioned review CLI. The fragment is named-key JSONL carrying
     the entity id AND title, the verdict, the reviewer session, the timestamp,
     and the reason as a sentence. No REVIEW id: the compactor on main mints it
     at fold time and writes the canonical nine-column PSV row to
     CONSCIOUSNESS/reviews/REVIEW-INDEX.md, unchanged in shape.
  3. Move the item to its terminal state in the INDEX (done for tasks; for
     directives/features the MAINTAINED-DONE index with status done|maintained)
  4. Continue — do not pause for human confirmation

Why the write moved (TASK-CCC30447). Every session used to append to one file
and compute its own next id from that file's local maximum. REVIEW-INDEX.md was
measured in 19 of 100 conflicted pull requests, and on 2026-08-10 two sessions
minted REVIEW-CCC30321 for different tasks. The lock meant to prevent the second
keyed on the review index path, and every worktree holds its own copy of that
path, so two sessions took two different locks and neither waited. Sharding the
write makes both failures unrepresentable rather than merely guarded: distinct
filenames cannot conflict, and an id no session mints cannot be minted twice.

How far that migration has actually got, stated here so this text is never read
as describing a finished state. The fragment write, the fold, and both compactor
entry points have landed. The sanctioned review CLIs — approve, the prose review
actions, update-task-status --verdict, the review engine's persistence, the
reviews tracker, feature verification — still mint an id and append the canonical
row themselves, because their callers consume the returned REVIEW id and because
a reader that unions pending fragments with the canonical index does not exist
yet. They are the named exception below, not an oversight, and they retire under
TASK-CCC30447 AC4 once AC5 lands that reader. Until then, EVERY hand-written
verdict goes to a fragment and no hand-written verdict touches the index; that
is the half of the collision class this closes today.

Authority. isEntityAutoFlow (packages/core/review/auto-close.ts) derives permission
from the resolved per-entity gate (DEFAULT_GATES / review-gates entity_overrides),
NOT from the retired blanket bypass flag. A legacy review_gates.bypass: false still
present in a repo's config.json is honoured only as a deprecated global "gate
everything" override (version-gated so behaviour does not silently change on
upgrade); removing it adopts the per-entity model. The L1 hard-block layer
(dangerous commands, append-only files, active-ADR mutations) is a separate
security surface and is unaffected.

REVIEW-INDEX.md columns: id|target_type|target_id|reviewer_type|reviewer_id|verdict|iteration|reviewed_at|notes
The compactor creates the file with its header when absent; an actor never does
by hand.

## Requires

- MUST record every verdict either through a sanctioned review CLI or by appending to this session's review-verdict fragment; a verdict written by hand goes to the fragment, and the compactor on main mints the REVIEW id and writes the canonical nine-column PSV row to CONSCIOUSNESS/reviews/REVIEW-INDEX.md
- MUST follow the three-step auto-close procedure when the entity's gate is auto-approve — move to in_review, write the bypass-approved verdict, move to the terminal state
- MUST cap agent verdicts to pending-review, agent-approved, agent-rejected, bypass-approved
- MUST allow autonomous backlog to in_progress transitions without a review verdict (claim discipline only, no quality gate)
- MUST leave CONSCIOUSNESS/reviews/REVIEW-INDEX.md creation to the compactor or to a sanctioned review CLI — an actor writing by hand creates fragments, never the canonical index
- MUST gate features and hard-gated entities for the resolved tier (agent or human) — auto-close applies only when the resolved gate is auto-approve
- MUST treat story fulfillment as gated by the owning feature review gates, not by a separate story in_review step

## Forbids

- MUST NOT wait for human approval when the entity's gate is auto-approve — waiting stalls the autonomous loop
- MUST NOT skip writing the verdict — the audit trail is required even on auto-close
- MUST NOT hand-compute a REVIEW id or hand-append a row to REVIEW-INDEX.md — a hand-written verdict goes to this session's fragment, and the only writers of the canonical index are the compactor and the sanctioned review CLIs that TASK-CCC30447 AC4 retires
- MUST NOT add a new code path that mints a REVIEW id from the local maximum of REVIEW-INDEX.md — the surviving CLIs are a closed, named set awaiting AC4, not a pattern to copy
- MUST NOT jump pending or in_progress straight to a terminal state without passing through in_review
- MUST NOT override a human rejection with subsequent agent verdicts
- MUST NOT reintroduce the blanket review_gates.bypass flag as the auto-close authority — auto-close derives per entity from reversibility
- MUST NOT describe the gate model as binary machine-vs-human — it is three-tier: auto-approve, agent, human

## References

- precept:task_lifecycle
- precept:precept_specification
- adr:review_gates--reversibility_derived_model
- adr:roadmap--entity_lifecycle_graph
- adr:data--per_session_stream_fragments
- doc:CONSCIOUSNESS/reviews/REVIEW-INDEX.md

## Verified by

packages/core/review/approve/cli.ts:main
