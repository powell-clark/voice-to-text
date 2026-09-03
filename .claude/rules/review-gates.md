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

That migration is now complete (TASK-CCC30678). The fragment write, the fold,
both compactor entry points, the union reader and the rendered review surface
have all landed, and the seven sanctioned review CLIs that used to mint their
own id — approve, the prose review actions, update-task-status --verdict, the
review engine's persistence, the reviews tracker, the gate primitives, feature
verification — now write id-less fragments through one shared writer. The
previously-named exception set is EMPTY: no session-side code path computes a
next REVIEW id, and a standing test plus a grep over the shipped dist keep it
that way.

Three things had to move with the write, and each would have been a silent
regression on its own. Iteration counting, prior-reviewer lookup and the
duplicate-verdict suppression all read REVIEW-INDEX.md, which after the split
no longer contains any verdict recorded since the last fold — so all three now
read the union. The verdict `evidence-found` was retired rather than carried
across: it sat outside the capped agent verdicts the fold accepts, so keeping
it would have stranded exactly those rows as permanently pending.

One consequence worth stating plainly. Remote-safe id reservation
(TASK-CCC30941) existed to stop two offline sessions minting the same id — a
correct fix for a problem that no longer exists, because a row carrying no id
cannot collide on one. That path is unreferenced and retires under
TASK-CCC30961.

A single, named exception to "no session-side minter survives" (TASK-CCC30698,
GitHub #1988/#1994). Four hand repairs of already-published REVIEW-INDEX id
collisions preceded this exception, each escalated to a human because no
sanctioned tool existed and hand-editing the canonical index is itself
forbidden below — an actor met a pre-push hook that demanded a repair and a
precept that forbade the only available form of it. `renumber-review.ts` and
`renumber-review-cli.ts` close that gap, and ONLY that gap: repairing rows
that ALREADY EXIST in a corrupted index, never allocating an id for a NEW
verdict. The distinction is not cosmetic — the repair's collapse pass needs no
new id at all, and its renumber pass only ever reassigns an id already
colliding on the canonical index, so it cannot manufacture the race this
precept exists to end; it can only end a race some other defect (the
compactor double-fold TASK-CCC30698's own acceptance criteria still owns) has
already caused. The exception is listed by filename in `entity_overrides`-style
scope, not granted to "repair tools" as a category — a future repair path
copies this narrative and re-earns its own listing, it does not inherit this
one.

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

- MUST record every verdict as a row in this session's review-verdict fragment, whether written by hand or through a review CLI; the compactor on main mints the REVIEW id and writes the canonical nine-column PSV row to CONSCIOUSNESS/reviews/REVIEW-INDEX.md
- MUST follow the three-step auto-close procedure when the entity's gate is auto-approve — move to in_review, write the bypass-approved verdict, move to the terminal state
- MUST cap agent verdicts to pending-review, agent-approved, agent-rejected, bypass-approved
- MUST allow autonomous backlog to in_progress transitions without a review verdict (claim discipline only, no quality gate)
- MUST leave CONSCIOUSNESS/reviews/REVIEW-INDEX.md creation and every write to it to the compactor alone — every other writer, actor or CLI, records fragments, except the named collision-repair exception below
- MUST gate features and hard-gated entities for the resolved tier (agent or human) — auto-close applies only when the resolved gate is auto-approve
- MUST treat story fulfillment as gated by the owning feature review gates, not by a separate story in_review step

## Forbids

- MUST NOT wait for human approval when the entity's gate is auto-approve — waiting stalls the autonomous loop
- MUST NOT skip writing the verdict — the audit trail is required even on auto-close
- MUST NOT hand-compute a REVIEW id or hand-append a row to REVIEW-INDEX.md — every verdict goes to a fragment, and the compactor is the sole writer of the canonical index
- MUST NOT add a code path that mints a REVIEW id from the local maximum of REVIEW-INDEX.md for a NEW verdict — no session-side minter for ordinary allocation survives, so there is no precedent to copy and a new one reopens the collision class; the sole named exception is packages/core/pgps/renumber-review.ts and renumber-review-cli.ts repairing an id that already collides on the published index, never allocating for a fresh verdict
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
- doc:CONSCIOUSNESS/tasks/backlog-task-item-details/TASK-CCC30698.md

## Verified by

packages/core/review/approve/cli.ts:main
