<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Task lifecycle

Every actor claims, advances, and releases work through the canonical four-state status graph: pending → in_progress → in_review → done. A reversible task whose per-entity review gate is auto-flow advances directly to done after in_review without waiting for human approval (auto-close path, ADR review_gates--reversibility_derived_model) — the normal path for autonomous operation; features and hard-gated entities wait for a human verdict. Only one task is in_progress per session at a time.

## Requires

- MUST resolve the task's per-entity review gate (review-gates precept) before closing — a reversible auto-flow task is authorised to mark done after moving to in_review; a hard-gated entity waits for a human verdict
- MUST claim only one task in_progress at a time per session
- MUST advance status linearly: pending → in_progress → in_review → done
- MUST release a task before claiming another
- MUST run /pgps and /universe (or the host's equivalent) before claiming a task to confirm it is unblocked, unclaimed, and not assigned to a human
- MUST commit and push the work that landed under a claim before transitioning to in_review
- MUST follow the auto-close path for a reversible auto-flow task: (1) update task status to in_review in the INDEX, (2) write a bypass-approved verdict row to CONSCIOUSNESS/reviews/REVIEW-INDEX.md, (3) update task status to done in the INDEX — all three steps in sequence
- MUST treat backlog → in_progress as an atomic claim that requires the same pre-checks as pending → in_progress

## Forbids

- MUST NOT wait for human approval when the task's per-entity gate is auto-flow — waiting stalls the loop unnecessarily
- MUST NOT skip writing the bypass-approved verdict row to REVIEW-INDEX.md — the audit trail is required even on auto-close
- MUST NOT claim tasks assigned to a human, blocked by an unfinished dependency, or actively worked by another session per active-sessions.json
- MUST NOT claim multiple tasks simultaneously
- MUST NOT context-switch between tasks without releasing the first to upcoming or in_review
- MUST NOT work without a claimed task — every shipping change must be traceable to a TASK-XXXX in_progress claim
- MUST NOT skip status states (no pending → in_review jumps; no backlog → done jumps); cancellations are operator-only

## References

- precept:review-gates
- precept:consciousness

## Verified by

packages/core/session/task-claiming/index.ts:claimTask
